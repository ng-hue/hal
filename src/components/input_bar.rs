use dioxus::prelude::*;

use crate::actions;
use crate::state::{use_app_state, use_registry};

/// Bottom bar: error banner + text box + send button.
/// Enter sends, Shift+Enter inserts a new line.
#[component]
pub fn InputBar() -> Element {
    let mut state = use_app_state();
    let registry = use_registry();
    let mut draft = use_signal(String::new);

    let streaming = *state.is_streaming.read();
    let error = state.last_error.read().clone();
    let can_send = !streaming && !draft.read().trim().is_empty();

    let submit = {
        let registry = registry.clone();
        move || {
            let text = draft.peek().clone();
            if !text.trim().is_empty() && !*state.is_streaming.peek() {
                draft.set(String::new());
                actions::send_message(state, registry.clone(), text);
            }
        }
    };
    let mut submit_on_enter = submit.clone();
    let mut submit_on_click = submit;

    rsx! {
        div { class: "input-area",
            if let Some(err) = error {
                div { class: "error-banner glass",
                    span { "{err}" }
                    button {
                        class: "btn btn--icon",
                        onclick: move |_| state.last_error.set(None),
                        "×"
                    }
                }
            }
            div { class: "input-bar glass",
                textarea {
                    class: "input-bar__text",
                    placeholder: "Message hal…",
                    rows: "1",
                    value: "{draft}",
                    oninput: move |evt| draft.set(evt.value()),
                    onkeydown: move |evt| {
                        if evt.key() == Key::Enter && !evt.modifiers().shift() {
                            evt.prevent_default();
                            submit_on_enter();
                        }
                    },
                }
                button {
                    class: "btn btn--send",
                    disabled: !can_send,
                    onclick: move |_| submit_on_click(),
                    if streaming { "…" } else { "Send" }
                }
            }
        }
    }
}
