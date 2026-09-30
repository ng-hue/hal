//! hal — entry point. Wires the global state, the orb background and the
//! chat shell together. Everything interesting lives in the modules below.

mod actions;
#[allow(dead_code)] // engine exposes API the UI will grow into
mod engine;
mod components;
mod state;
mod voice;

use dioxus::prelude::*;

use components::{ChatView, InputBar, ModeToggle, Orb, Sidebar};
use state::Mode;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

        let window = WindowBuilder::new()
            .with_title("hal")
            .with_inner_size(LogicalSize::new(1280.0, 820.0))
            .with_min_inner_size(LogicalSize::new(720.0, 480.0));

        dioxus::LaunchBuilder::desktop()
            .with_cfg(
                Config::new()
                    .with_window(window)
                    .with_menu(None::<dioxus::desktop::muda::Menu>),
            )
            .launch(App);
    }

    #[cfg(not(feature = "desktop"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let state = state::use_app_state_provider();
    voice::use_voice_engine(state);

    // The shell stays mounted in voice mode (just faded out) so drafts,
    // scroll position and in-flight replies survive the toggle.
    let shell_class = match *state.mode.read() {
        Mode::Manual => "shell",
        Mode::Voice => "shell shell--hidden",
    };

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        document::Title { "hal" }

        Orb {}
        ModeToggle {}

        div { class: "{shell_class}",
            Sidebar {}
            main { class: "chat-pane",
                ChatView {}
                InputBar {}
            }
        }
    }
}
