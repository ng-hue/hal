use dioxus::prelude::*;

use super::ModelPicker;
use crate::state::use_app_state;

/// Left column: brand, new-chat button, model picker, conversation history.
#[component]
pub fn Sidebar() -> Element {
    let mut state = use_app_state();
    let active = *state.active_id.read();

    // Snapshot just what we render (newest first) so no borrow is held.
    let items: Vec<_> = state
        .conversations
        .read()
        .iter()
        .rev()
        .map(|c| (c.id, c.title.clone()))
        .collect();

    rsx! {
        aside { class: "sidebar glass",
            div { class: "sidebar__header",
                span { class: "brand",
                    span { class: "brand__dot" }
                    "hal"
                }
                button {
                    class: "btn btn--ghost",
                    onclick: move |_| {
                        state.new_conversation();
                    },
                    "+ New chat"
                }
            }

            ModelPicker {}

            div { class: "section-label", "Conversations" }
            nav { class: "conv-list",
                if items.is_empty() {
                    p { class: "muted small", "No conversations yet." }
                }
                for (id, title) in items {
                    div {
                        key: "{id}",
                        class: if active == Some(id) { "conv-item conv-item--active" } else { "conv-item" },
                        onclick: move |_| state.open_conversation(id),
                        span { class: "conv-item__title", "{title}" }
                        button {
                            class: "conv-item__delete",
                            title: "Delete conversation",
                            onclick: move |evt| {
                                evt.stop_propagation();
                                state.delete_conversation(id);
                            },
                            "×"
                        }
                    }
                }
            }
        }
    }
}
