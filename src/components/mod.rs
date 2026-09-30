//! UI components. One file per component; each reads what it needs from
//! `AppState` instead of taking long prop lists.

mod chat_view;
mod input_bar;
mod message_bubble;
mod mode_toggle;
mod model_picker;
mod orb;
mod sidebar;

pub use chat_view::ChatView;
pub use input_bar::InputBar;
pub use message_bubble::MessageBubble;
pub use mode_toggle::ModeToggle;
pub use model_picker::ModelPicker;
pub use orb::Orb;
pub use sidebar::Sidebar;
