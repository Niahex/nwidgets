use gpui::EventEmitter;

pub const DEFAULT_URL: &str = "https://gemini.google.com/app";

pub struct ChatToggled;
pub struct ChatPinToggled;
pub struct ChatNavigate {
    pub url: String,
}

pub trait ChatEvents: EventEmitter<ChatToggled> + EventEmitter<ChatPinToggled> + EventEmitter<ChatNavigate> {}
