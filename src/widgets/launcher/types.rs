use crate::services::system::clipboard::ClipboardEntry;
use gpui::EventEmitter;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplicationInfo {
    pub name: String,
    pub name_lower: String,
    pub exec: String,
    pub icon: Option<String>,
    pub icon_path: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_mb: f32,
}

#[derive(Clone)]
pub struct LauncherToggled;

pub enum SearchResultType {
    Application(usize),
    Calculation(String),
    Process(ProcessInfo),
    Clipboard(ClipboardEntry),
}

#[derive(Clone)]
pub enum SearchResult {
    Application(ApplicationInfo),
    Calculation(String),
    Process(ProcessInfo),
    Clipboard(ClipboardEntry),
}

pub trait LauncherEvents: EventEmitter<LauncherToggled> {}
