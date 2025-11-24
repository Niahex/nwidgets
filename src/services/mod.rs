pub mod applications;
pub mod bluetooth;
pub mod capslock;
pub mod clipboard;
pub mod hyprland;
pub mod notifications;
pub mod numlock;
pub mod osd;
pub mod pipewire;
pub mod pomodoro;
pub mod pin_controller;
// pub mod speech;
pub mod systray;
// pub mod transcription;

pub use applications::ApplicationsService;
pub use notifications::NotificationService;
pub use pin_controller::PinController;
// pub use speech::SpeechRecognitionService;
// pub use transcription::{
//     receive_transcription_events, TranscriptionEvent, TranscriptionEventService,
// };
