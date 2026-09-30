use dioxus::prelude::*;
use crate::engine::Role;

/// One chat bubble. `pending` shows a typing indicator until text arrives.
#[component]
pub fn MessageBubble(role: Role, content: String, #[props(default)] pending: bool) -> Element {
    let (class, label) = match role {
        Role::User => ("msg msg--user", "You"),
        Role::Assistant => ("msg msg--assistant glass", "hal"),
        Role::System => ("msg msg--system", "system"),
    };

    rsx! {
        div { class: "{class}",
            div { class: "msg__label", "{label}" }
            if pending && content.is_empty() {
                div { class: "typing", span {} span {} span {} }
            } else {
                div { class: "msg__body", "{content}" }
            }
        }
    }
}
