//! `engine` — everything that is *not* UI. Keep Dioxus out of this folder
//! so it can later be split into its own crate (for a server or iPad app).
//!
//! The desktop app, a future iPad app, and a future home server will all use
//! this, so nothing in here may know about Dioxus, windows, or pixels.
//!
//! * [`conversation`] — plain data: messages and conversations.
//! * [`llm`] — the [`llm::LlmProvider`] trait plus concrete providers
//!   (Ollama, Echo) and a [`llm::ProviderRegistry`] to swap between them.

pub mod conversation;
pub mod llm;

pub use conversation::{Conversation, ConversationId, Message, Role};
