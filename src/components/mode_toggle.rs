use dioxus::prelude::*;

use crate::state::{use_app_state, Mode};

/// Floating button (top-right) that switches between chat and voice mode.
#[component]
pub fn ModeToggle() -> Element {
    let mut state = use_app_state();
    let voice = *state.mode.read() == Mode::Voice;

    let (class, label, title) = if voice {
        ("mode-toggle glass mode-toggle--voice", "Back to chat", "Show the chat interface")
    } else {
        ("mode-toggle glass", "Voice mode", "Hide the chat and talk to hal")
    };

    rsx! {
        button {
            class: "{class}",
            title: "{title}",
            onclick: move |_| state.toggle_mode(),
            span { class: "mode-toggle__dot" }
            "{label}"
        }
    }
}
