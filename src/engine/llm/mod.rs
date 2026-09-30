//! Model abstraction. **The rest of the app only ever talks to
//! [`LlmProvider`]** — it never knows whether the model is Ollama, a cloud
//! API, or something running in-process. Adding a model = implementing this
//! trait and registering it in the [`ProviderRegistry`].

mod echo;
mod ollama;
mod registry;

pub use echo::EchoProvider;
pub use ollama::OllamaProvider;
pub use registry::ProviderRegistry;

use std::fmt;

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::engine::Message;

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("could not parse model response: {0}")]
    Parse(String),
    #[error("no provider named `{0}` is registered")]
    UnknownProvider(String),
    #[error("{0}")]
    Other(String),
}

/// A stream of text chunks ("tokens") as the model produces them.
pub type TokenStream = BoxStream<'static, Result<String, LlmError>>;

#[derive(Debug, Clone)]
pub struct ChatRequest {
    /// Model name as the provider knows it, e.g. `llama3.2`.
    pub model: String,
    /// Full history, oldest first.
    pub messages: Vec<Message>,
}

/// Anything that can chat. Object-safe on purpose (`Arc<dyn LlmProvider>`),
/// so providers can be chosen at runtime from a dropdown.
pub trait LlmProvider: Send + Sync {
    /// Stable machine id, e.g. `"ollama"`. Used as part of [`ModelRef`].
    fn id(&self) -> &str;
    /// Human-readable name for the UI.
    fn display_name(&self) -> &str;
    /// Models this provider can serve right now.
    fn list_models(&self) -> BoxFuture<'static, Result<Vec<String>, LlmError>>;
    /// Start a streamed chat completion.
    fn chat(&self, request: ChatRequest) -> TokenStream;
}

/// Points at one model on one provider. This is what the model picker stores.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelRef {
    pub provider: String,
    pub model: String,
}

impl ModelRef {
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self { provider: provider.into(), model: model.into() }
    }

    /// Single-string form (`provider::model`), handy for `<select>` values.
    pub fn key(&self) -> String {
        format!("{}::{}", self.provider, self.model)
    }

    pub fn from_key(key: &str) -> Option<Self> {
        let (provider, model) = key.split_once("::")?;
        Some(Self::new(provider, model))
    }
}

impl fmt::Display for ModelRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} · {}", self.model, self.provider)
    }
}
