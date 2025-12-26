pub mod bluetooth;
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

// Re-export services for backward compatibility
pub use notifications::NotificationService;
