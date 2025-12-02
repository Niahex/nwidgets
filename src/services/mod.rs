pub mod bluetooth;
pub mod chat;
pub mod clipboard;
pub mod hyprland;
pub mod lock_state;
pub mod mpris;
pub mod network;
pub mod notifications;
pub mod osd;
pub mod pipewire;
pub mod pomodoro;
pub mod stt;
pub mod systray;

// Re-export lock state services for backward compatibility
pub use chat::ChatStateService;
pub use lock_state::{CapsLockService, NumLockService};
pub use mpris::MprisService;
pub use notifications::NotificationService;
pub use stt::SttService;
