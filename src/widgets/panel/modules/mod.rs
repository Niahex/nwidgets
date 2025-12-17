mod active_window;
mod audio;
mod datetime;
mod mpris;
mod network;
mod pomodoro;
mod systray;
mod workspaces;

pub use active_window::ActiveWindowModule;
pub use audio::AudioModule;
pub use datetime::DateTimeModule;
pub use mpris::MprisModule;
pub use network::NetworkModule;
pub use pomodoro::PomodoroModule;
pub use systray::SystrayModule;
pub use workspaces::WorkspacesModule;
