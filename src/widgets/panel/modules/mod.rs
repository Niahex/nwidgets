pub mod active_window;
pub mod audio;
pub mod bluetooth;
pub mod datetime;
pub mod network;
pub mod pomodoro;
pub mod systray;
pub mod workspaces;

// Re-export audio module types for backward compatibility
pub use audio::{AudioDeviceType, AudioModule, SinkModule, SourceModule};
