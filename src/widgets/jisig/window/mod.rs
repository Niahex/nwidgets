pub mod event_handlers;
pub mod window_manager;

pub use event_handlers::{on_fullscreen, on_toggle, on_workspace_change};
pub use window_manager::open;
