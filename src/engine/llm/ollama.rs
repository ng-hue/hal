//! Ollama provider — free, local models (https://ollama.com).
//!
//! Uses Ollama's native API:
//! * `GET  /api/tags` — installed models
//! * `POST /api/chat` with `"stream": true` — newline-delimited JSON chunks

use futures::{future::BoxFuture, stream, Stream, StreamExt, TryStreamExt};
use serde::Deserialize;

use super::{ChatRequest, LlmError, LlmProvider, TokenStream};

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaProvider {
    pub const DEFAULT_URL: &'static str = "http://localhost:11434";

    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new(Self::DEFAULT_URL)
    }
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    name: String,
}

#[derive(Deserialize)]
struct ChatChunk {
    #[serde(default)]
    message: Option<ChunkMessage>,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct ChunkMessage {
    #[serde(default)]
    content: String,
}

impl LlmProvider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama"
    }

    fn display_name(&self) -> &str {
        "Ollama (local)"
    }

    fn list_models(&self) -> BoxFuture<'static, Result<Vec<String>, LlmError>> {
        let client = self.client.clone();
        let url = format!("{}/api/tags", self.base_url);
        Box::pin(async move {
            let tags: TagsResponse =
                client.get(url).send().await?.error_for_status()?.json().await?;
            Ok(tags.models.into_iter().map(|m| m.name).collect())
        })
    }

    fn chat(&self, request: ChatRequest) -> TokenStream {
        let client = self.client.clone();
        let url = format!("{}/api/chat", self.base_url);
        let body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "stream": true,
        });

        let connect = async move {
            let resp = client.post(url).json(&body).send().await?.error_for_status()?;
            Ok::<_, LlmError>(parse_ndjson(Box::pin(resp.bytes_stream())))
        };

        stream::once(connect).try_flatten().boxed()
    }
}

/// Turns a raw byte stream of newline-delimited JSON into text chunks.
/// Network chunks can split a JSON line in half, so we buffer until `\n`.
fn parse_ndjson<S>(bytes: S) -> impl Stream<Item = Result<String, LlmError>> + Send + 'static
where
    S: Stream<Item = reqwest::Result<bytes::Bytes>> + Send + Unpin + 'static,
{
    parse_lines(bytes.map_err(LlmError::from))
}

fn parse_lines<S>(bytes: S) -> impl Stream<Item = Result<String, LlmError>> + Send + 'static
where
    S: Stream<Item = Result<bytes::Bytes, LlmError>> + Send + Unpin + 'static,
{
    stream::unfold((bytes, Vec::<u8>::new(), false), |(mut bytes, mut buf, finished)| async move {
        if finished {
            return None;
        }
        loop {
            if let Some(pos) = buf.iter().position(|b| *b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let line = &line[..line.len() - 1];
                if line.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }
                let item = match serde_json::from_slice::<ChatChunk>(line) {
                    Ok(ChatChunk { error: Some(err), .. }) => (Err(LlmError::Other(err)), true),
                    Ok(chunk) => (Ok(chunk.message.map(|m| m.content).unwrap_or_default()), chunk.done),
                    Err(e) => (Err(LlmError::Parse(e.to_string())), true),
                };
                return Some((item.0, (bytes, buf, item.1)));
            }
            match bytes.next().await {
                Some(Ok(chunk)) => buf.extend_from_slice(&chunk),
                Some(Err(e)) => return Some((Err(e), (bytes, buf, true))),
                None => return None,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lines_split_across_network_chunks() {
        let raw = [
            "{\"message\":{\"role\":\"assistant\",\"content\":\"Hel\"},\"done\":false}\n{\"mess",
            "age\":{\"role\":\"assistant\",\"content\":\"lo\"},\"done\":false}\n",
            "{\"done\":true}\n",
        ];
        let input = stream::iter(raw.map(|s| Ok(bytes::Bytes::from(s))));
        let out: Vec<String> = futures::executor::block_on(
            parse_lines(input).map(|r| r.unwrap()).collect::<Vec<_>>(),
        );
        assert_eq!(out.concat(), "Hello");
    }

    #[test]
    fn surfaces_ollama_errors() {
        let input = stream::iter([Ok(bytes::Bytes::from("{\"error\":\"model not found\"}\n"))]);
        let out: Vec<_> = futures::executor::block_on(parse_lines(input).collect::<Vec<_>>());
        assert!(matches!(&out[0], Err(LlmError::Other(m)) if m == "model not found"));
    }
}
