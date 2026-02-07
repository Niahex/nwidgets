pub mod service;
pub mod types;
pub mod widget;
pub mod window;

pub use service::JisigService;
pub use types::JisigToggled;
pub use widget::JisigWidget;
pub use window::{on_fullscreen, on_toggle, on_workspace_change, open};
