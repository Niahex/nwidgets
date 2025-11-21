pub mod bluetooth;
pub mod capslock;
pub mod clipboard;
pub mod hyprland;
// pub mod notifications;
pub mod numlock;
pub mod osd;
pub mod pipewire;
pub mod pomodoro;
// pub mod speech;
pub mod systray;
// pub mod transcription;

pub use bluetooth::{BluetoothService, BluetoothState};
pub use capslock::CapsLockService;
pub use clipboard::ClipboardService;
pub use hyprland::HyprlandService;
pub use systray::{SystemTrayService, TrayItem};
pub use pipewire::{PipeWireService, AudioState};
pub use pomodoro::{PomodoroService, PomodoroState};
pub use numlock::NumLockService;
pub use osd::{receive_osd_events, OsdEvent, OsdEventService};
// pub use notifications::{Notification, NotificationManager, NotificationService};
// pub use speech::SpeechRecognitionService;
// pub use transcription::{
//     receive_transcription_events, TranscriptionEvent, TranscriptionEventService,
// };
