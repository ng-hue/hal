//! Conversation data types. Serializable so they can be saved to disk or
//! sent over the network later without changes.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::llm::ModelRef;

/// Who wrote a message. Serialized lowercase to match the OpenAI/Ollama format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into() }
    }
}

pub type ConversationId = Uuid;

pub const DEFAULT_TITLE: &str = "New chat";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conversation {
    pub id: ConversationId,
    pub title: String,
    pub messages: Vec<Message>,
    /// The model last used in this conversation, so switching back restores it.
    pub model: Option<ModelRef>,
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: DEFAULT_TITLE.to_string(),
            messages: Vec::new(),
            model: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Names the conversation after the first user message (first ~40 chars).
    pub fn set_title_from_first_message(&mut self) {
        if self.title != DEFAULT_TITLE {
            return;
        }
        if let Some(first) = self.messages.iter().find(|m| m.role == Role::User) {
            let text = first.content.trim();
            let mut title: String = text.chars().take(40).collect();
            if text.chars().count() > 40 {
                title.push('…');
            }
            if !title.is_empty() {
                self.title = title;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_comes_from_first_user_message() {
        let mut c = Conversation::new();
        c.messages.push(Message::user("Open the pod bay doors please, it is getting cold out here"));
        c.set_title_from_first_message();
        assert!(c.title.starts_with("Open the pod bay doors"));
        assert!(c.title.ends_with('…'));
    }
}
