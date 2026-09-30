//! Things the app *does*. Components stay dumb: they call these functions,
//! these functions update state and talk to `hal-core`.

use dioxus::prelude::*;
use futures::StreamExt;
use crate::engine::{ConversationId, Message};

use crate::state::{AppState, Registry};

/// Looks up models from every provider and picks a default if needed.
pub fn refresh_models(mut state: AppState, registry: Registry) {
    spawn(async move {
        let (models, errors) = registry.0.discover_models().await;

        let still_valid = state
            .selected_model
            .peek()
            .as_ref()
            .is_some_and(|m| models.contains(m));
        if !still_valid {
            state.selected_model.set(models.first().cloned());
        }
        state.models.set(models);

        if !errors.is_empty() {
            let names: Vec<&str> = errors.iter().map(|(name, _)| name.as_str()).collect();
            state.last_error.set(Some(format!(
                "Couldn't reach {}. Start it, then press ↻ next to the model list.",
                names.join(", ")
            )));
        }
    });
}

/// Sends `text` in the active conversation and streams the reply into it.
pub fn send_message(mut state: AppState, registry: Registry, text: String) {
    let text = text.trim().to_string();
    if text.is_empty() || *state.is_streaming.peek() {
        return;
    }
    let Some(model) = state.selected_model.peek().clone() else {
        state.last_error.set(Some("Pick a model on the left first.".into()));
        return;
    };

    let active = *state.active_id.peek();
    let conv_id = match active {
        Some(id) => id,
        None => state.new_conversation(),
    };

    // Add the user's message and an empty assistant message to fill in.
    let history = {
        let mut convs = state.conversations.write();
        let Some(conv) = convs.iter_mut().find(|c| c.id == conv_id) else {
            return;
        };
        conv.model = Some(model.clone());
        conv.messages.push(Message::user(text));
        conv.set_title_from_first_message();
        let history = conv.messages.clone();
        conv.messages.push(Message::assistant(""));
        history
    };

    state.last_error.set(None);
    state.is_streaming.set(true);

    spawn(async move {
        match registry.0.chat(&model, history) {
            Ok(mut tokens) => {
                while let Some(chunk) = tokens.next().await {
                    match chunk {
                        Ok(text) => append_to_reply(state, conv_id, &text),
                        Err(e) => {
                            state.last_error.set(Some(e.to_string()));
                            break;
                        }
                    }
                }
            }
            Err(e) => state.last_error.set(Some(e.to_string())),
        }
        drop_empty_reply(state, conv_id);
        state.is_streaming.set(false);
    });
}

fn append_to_reply(mut state: AppState, conv_id: ConversationId, text: &str) {
    if text.is_empty() {
        return;
    }
    let mut convs = state.conversations.write();
    if let Some(msg) = convs
        .iter_mut()
        .find(|c| c.id == conv_id)
        .and_then(|c| c.messages.last_mut())
    {
        msg.content.push_str(text);
    }
}

/// If the model failed before saying anything, remove the blank bubble.
fn drop_empty_reply(mut state: AppState, conv_id: ConversationId) {
    let mut convs = state.conversations.write();
    if let Some(conv) = convs.iter_mut().find(|c| c.id == conv_id) {
        if conv.messages.last().is_some_and(|m| m.content.is_empty()) {
            conv.messages.pop();
        }
    }
}
