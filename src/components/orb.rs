//! The animated background. Pure CSS layers (see `assets/main.css`, section
//! "Orb") driven by two inputs:
//! * the mode class (`orb--awake` / `orb--dim`)
//! * the `--level` CSS variable, fed from `AppState::voice_level`
//!
//! To upgrade to a shader later, replace the inner `div`s with a canvas —
//! nothing else in the app needs to change.

use dioxus::prelude::*;

use crate::state::{use_app_state, Mode};

#[component]
pub fn Orb() -> Element {
    let state = use_app_state();
    let awake = *state.mode.read() == Mode::Voice;
    let thinking = *state.is_streaming.read();
    let level = *state.voice_level.read();

    let class = format!(
        "orb-layer {} {}",
        if awake { "orb--awake" } else { "orb--dim" },
        if thinking { "orb--thinking" } else { "" },
    );

    rsx! {
        div { class: "{class}", style: "--level: {level:.3};",
            div { class: "orb",
                div { class: "orb__haze" }
                div { class: "orb__glow" }
                div { class: "orb__ring" }
                div { class: "orb__core" }
                div { class: "orb__glint" }
            }
            div { class: "orb__scanlines" }
            if awake {
                div { class: "orb__status", "Listening…" }
            }
        }
    }
}
