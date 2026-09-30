use dioxus::prelude::*;

use super::MessageBubble;
use crate::state::use_app_state;

/// Middle column: the scrolling message list.
///
/// Auto-scroll trick: the scroller uses `flex-direction: column-reverse`,
/// which pins it to the bottom as new text streams in — no JS needed.
#[component]
pub fn ChatView() -> Element {
    let state = use_app_state();
    let conversation = state.active_conversation();
    let streaming = *state.is_streaming.read();

    let body = match conversation {
        Some(conv) if !conv.messages.is_empty() => {
            let last = conv.messages.len() - 1;
            rsx! {
                for (i, msg) in conv.messages.into_iter().enumerate() {
                    MessageBubble {
                        key: "{i}",
                        role: msg.role,
                        content: msg.content,
                        pending: streaming && i == last,
                    }
                }
            }
        }
        _ => rsx! {
            div { class: "empty-state",
                h1 { "What can I do for you?" }
                p { class: "muted", "Pick a model on the left, or switch to voice mode." }
            }
        },
    };

    rsx! {
        section { class: "chat-scroll",
            div { class: "chat-scroll__inner", {body} }
        }
    }
}
