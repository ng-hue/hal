//! Holds every registered provider and routes requests by [`ModelRef`].

use std::sync::Arc;

use futures::future::join_all;

use super::{ChatRequest, LlmError, LlmProvider, ModelRef, TokenStream};
use crate::engine::Message;

#[derive(Default, Clone)]
pub struct ProviderRegistry {
    providers: Vec<Arc<dyn LlmProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style registration: `ProviderRegistry::new().with(OllamaProvider::default())`.
    pub fn with(mut self, provider: impl LlmProvider + 'static) -> Self {
        self.register(Arc::new(provider));
        self
    }

    pub fn register(&mut self, provider: Arc<dyn LlmProvider>) {
        self.providers.push(provider);
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn LlmProvider>> {
        self.providers.iter().find(|p| p.id() == id).cloned()
    }

    pub fn providers(&self) -> &[Arc<dyn LlmProvider>] {
        &self.providers
    }

    /// Asks every provider for its models, in registration order.
    /// Providers that fail (e.g. Ollama not running) are returned as
    /// `(display_name, error)` pairs instead of failing the whole call.
    pub async fn discover_models(&self) -> (Vec<ModelRef>, Vec<(String, LlmError)>) {
        let results = join_all(self.providers.iter().map(|p| {
            let id = p.id().to_string();
            let name = p.display_name().to_string();
            let fut = p.list_models();
            async move { (id, name, fut.await) }
        }))
        .await;

        let mut models = Vec::new();
        let mut errors = Vec::new();
        for (id, name, result) in results {
            match result {
                Ok(list) => models.extend(list.into_iter().map(|m| ModelRef::new(id.clone(), m))),
                Err(e) => errors.push((name, e)),
            }
        }
        (models, errors)
    }

    pub fn chat(&self, model: &ModelRef, messages: Vec<Message>) -> Result<TokenStream, LlmError> {
        let provider = self
            .get(&model.provider)
            .ok_or_else(|| LlmError::UnknownProvider(model.provider.clone()))?;
        Ok(provider.chat(ChatRequest { model: model.model.clone(), messages }))
    }
}
