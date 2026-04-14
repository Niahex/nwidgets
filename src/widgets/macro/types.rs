use gpui::SharedString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub id: Uuid,
    pub name: String,
    pub app_class: Option<String>,
    pub actions: Vec<MacroAction>,
    pub created_at: u64,
}

impl Macro {
    pub fn new(name: String, app_class: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            app_class,
            actions: Vec::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn duration_ms(&self) -> u64 {
        self.actions.last().map(|a| a.timestamp_ms).unwrap_or(0)
    }

    pub fn action_count(&self) -> usize {
        self.actions.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroAction {
    pub timestamp_ms: u64,
    pub action_type: ActionType,
    pub click_zone: Option<ClickZone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickZone {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    KeyPress(u32),
    KeyRelease(u32),
    MouseClick(MacroMouseButton),
    Delay(u64),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MacroMouseButton {
    Left,
    Right,
    Middle,
}

impl ActionType {
    pub fn display_name(&self) -> SharedString {
        match self {
            ActionType::KeyPress(code) => format!("Key Press ({})", code).into(),
            ActionType::KeyRelease(code) => format!("Key Release ({})", code).into(),
            ActionType::MouseClick(btn) => format!("Mouse Click ({:?})", btn).into(),
            ActionType::Delay(ms) => format!("Delay ({}ms)", ms).into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MacroToggled;

#[derive(Clone, Debug)]
pub struct MacroRecordingChanged {
    pub recording: bool,
}

#[derive(Clone, Debug)]
pub struct MacroPlayingChanged {
    pub playing: bool,
    pub macro_id: Option<Uuid>,
}

#[derive(Clone, Debug)]
pub struct MacroListChanged;
