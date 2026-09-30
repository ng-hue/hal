//! Offline test provider. Streams your message back word by word so you can
//! build and test the UI without any model running.

use std::time::Duration;

use futures::{stream, StreamExt};
use futures_timer::Delay;

use super::{ChatRequest, LlmError, LlmProvider, TokenStream};
use crate::engine::Role;

#[derive(Debug, Default, Clone, Copy)]
pub struct EchoProvider;

impl LlmProvider for EchoProvider {
    fn id(&self) -> &str {
        "echo"
    }

    fn display_name(&self) -> &str {
        "Echo (offline test)"
    }

    fn list_models(&self) -> futures::future::BoxFuture<'static, Result<Vec<String>, LlmError>> {
        Box::pin(async { Ok(vec!["echo".to_string()]) })
    }

    fn chat(&self, request: ChatRequest) -> TokenStream {
        let last = request
            .messages
            .iter()
            .rev()
            .find(|m| m.role == Role::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let reply = format!(
            "I heard: \"{last}\". I'm the Echo test model — start Ollama and pick a real model on the left to get actual answers."
        );
        let words: Vec<String> = reply.split_inclusive(' ').map(String::from).collect();

        stream::iter(words)
            .then(|w| async move {
                Delay::new(Duration::from_millis(35)).await;
                Ok(w)
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Message;

    #[test]
    fn echoes_last_user_message() {
        let req = ChatRequest { model: "echo".into(), messages: vec![Message::user("hello")] };
        let chunks: Vec<String> = futures::executor::block_on(
            EchoProvider.chat(req).map(|r| r.unwrap()).collect::<Vec<_>>(),
        );
        assert!(chunks.concat().contains("\"hello\""));
    }
}
