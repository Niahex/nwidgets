pub mod service;
pub mod types;
pub mod widget;
pub mod window;

pub use service::ChatService;
pub use types::{ChatNavigate, ChatPinToggled, ChatToggled};
pub use widget::ChatWidget;
pub use window::{on_fullscreen, on_navigate, on_toggle, on_workspace_change, open};
