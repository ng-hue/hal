//! Global app state. One `AppState` (a bundle of signals) is provided at the
//! root and read by any component with `use_app_state()`.
//!
//! Signals are `Copy`, so `AppState` is too — pass it into closures and
//! async tasks freely. Reading a signal in a component subscribes that
//! component; writing it re-renders only the subscribers.

use std::sync::Arc;

use dioxus::prelude::*;
use crate::engine::llm::{EchoProvider, ModelRef, OllamaProvider, ProviderRegistry};
use crate::engine::{Conversation, ConversationId};

use crate::actions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Chat UI visible, orb dimmed in the background.
    Manual,
    /// Chat UI hidden, orb awake and listening.
    Voice,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub mode: Signal<Mode>,
    pub conversations: Signal<Vec<Conversation>>,
    pub active_id: Signal<Option<ConversationId>>,
    pub models: Signal<Vec<ModelRef>>,
    pub selected_model: Signal<Option<ModelRef>>,
    pub is_streaming: Signal<bool>,
    /// 0.0–1.0 loudness driving the orb. Fed by `voice.rs`.
    pub voice_level: Signal<f32>,
    pub last_error: Signal<Option<String>>,
}

/// Shared handle to the model registry (not a signal — it never changes
/// after startup).
#[derive(Clone)]
pub struct Registry(pub Arc<ProviderRegistry>);

/// **This is where models are registered.** Order matters: the first model
/// found becomes the default selection.
fn build_registry() -> ProviderRegistry {
    ProviderRegistry::new()
        .with(OllamaProvider::default())
        .with(EchoProvider)
}

/// Call once, in the root component.
pub fn use_app_state_provider() -> AppState {
    let registry = use_context_provider(|| Registry(Arc::new(build_registry())));
    let state = use_context_provider(|| AppState {
        mode: Signal::new(Mode::Manual),
        conversations: Signal::new(Vec::new()),
        active_id: Signal::new(None),
        models: Signal::new(Vec::new()),
        selected_model: Signal::new(None),
        is_streaming: Signal::new(false),
        voice_level: Signal::new(0.0),
        last_error: Signal::new(None),
    });

    use_hook(|| actions::refresh_models(state, registry.clone()));
    state
}

pub fn use_app_state() -> AppState {
    use_context()
}

pub fn use_registry() -> Registry {
    use_context()
}

impl AppState {
    pub fn toggle_mode(&mut self) {
        let next = match *self.mode.peek() {
            Mode::Manual => Mode::Voice,
            Mode::Voice => Mode::Manual,
        };
        self.mode.set(next);
    }

    /// Clone of the open conversation (subscribes the caller to changes).
    pub fn active_conversation(&self) -> Option<Conversation> {
        let id = (*self.active_id.read())?;
        self.conversations.read().iter().find(|c| c.id == id).cloned()
    }

    /// Opens a fresh conversation — or reuses the current one if it's still empty.
    pub fn new_conversation(&mut self) -> ConversationId {
        if let Some(id) = *self.active_id.peek() {
            if self.conversations.peek().iter().any(|c| c.id == id && c.is_empty()) {
                return id;
            }
        }
        let conv = Conversation::new();
        let id = conv.id;
        self.conversations.write().push(conv);
        self.active_id.set(Some(id));
        id
    }

    pub fn open_conversation(&mut self, id: ConversationId) {
        let model = self
            .conversations
            .peek()
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.model.clone());
        if let Some(model) = model {
            self.selected_model.set(Some(model));
        }
        self.active_id.set(Some(id));
    }

    pub fn delete_conversation(&mut self, id: ConversationId) {
        self.conversations.write().retain(|c| c.id != id);
        if *self.active_id.peek() == Some(id) {
            let last = self.conversations.peek().last().map(|c| c.id);
            self.active_id.set(last);
        }
    }
}
