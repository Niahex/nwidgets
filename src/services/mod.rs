pub mod applications;
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
// pub mod speech;
pub mod systray;
// pub mod transcription;

// Re-export lock state services for backward compatibility
pub use lock_state::{CapsLockService, NumLockService};

pub use applications::ApplicationsService;
pub use chat::ChatStateService;
pub use mpris::MprisService;
pub use notifications::NotificationService;
// pub use speech::SpeechRecognitionService;
// pub use transcription::{
//     receive_transcription_events, TranscriptionEvent, TranscriptionEventService,
// };
