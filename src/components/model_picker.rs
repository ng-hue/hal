use dioxus::prelude::*;
use crate::engine::llm::ModelRef;

use crate::actions;
use crate::state::{use_app_state, use_registry};

/// Dropdown of every model from every registered provider.
#[component]
pub fn ModelPicker() -> Element {
    let mut state = use_app_state();
    let registry = use_registry();

    let models = state.models.read().clone();
    let no_models = models.is_empty();
    let selected = state.selected_model.read().as_ref().map(ModelRef::key).unwrap_or_default();

    rsx! {
        div { class: "section-label", "Model" }
        div { class: "model-picker",
            select {
                class: "select",
                value: "{selected}",
                disabled: no_models,
                onchange: move |evt| {
                    if let Some(model) = ModelRef::from_key(&evt.value()) {
                        state.selected_model.set(Some(model));
                    }
                },
                if no_models {
                    option { value: "", "Looking for models…" }
                }
                for model in models {
                    option {
                        key: "{model.key()}",
                        value: "{model.key()}",
                        selected: model.key() == selected,
                        "{model}"
                    }
                }
            }
            button {
                class: "btn btn--icon",
                title: "Refresh model list",
                onclick: move |_| actions::refresh_models(state, registry.clone()),
                "↻"
            }
        }
    }
}
