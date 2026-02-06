pub mod modules;
pub mod widget;
pub mod window;

pub use modules::{
    ActiveWindowModule, BluetoothModule, DateTimeModule, MprisModule, NetworkModule,
    PomodoroModule, SinkModule, SourceModule, SystrayModule, WorkspacesModule,
};
pub use widget::Panel;
pub use window::open;
