mod active_window;
mod audio_volume;
mod bluetooth;
mod datetime;
pub mod mpris;
mod network;
pub mod pomodoro;
pub mod systray;
mod workspaces;

pub use active_window::ActiveWindowModule;
pub use audio_volume::AudioVolumeModule;
pub use bluetooth::BluetoothModule;
pub use datetime::DateTimeModule;
pub use mpris::{MprisModule, MprisService};
pub use network::NetworkModule;
pub use pomodoro::{PomodoroModule, PomodoroService};
pub use systray::SystrayWidget;
pub use workspaces::WorkspacesModule;
