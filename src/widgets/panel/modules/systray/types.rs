use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayItem {
    pub service: String,
    pub object_path: String,
    pub id: String,
    pub title: String,
    pub status: TrayStatus,
    pub category: TrayCategory,
    pub icon_name: Option<String>,
    pub icon_pixmap: Option<Vec<TrayIcon>>,
    pub attention_icon_name: Option<String>,
    pub menu_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrayStatus {
    Passive,
    Active,
    NeedsAttention,
}

impl Default for TrayStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrayCategory {
    ApplicationStatus,
    Communications,
    SystemServices,
    Hardware,
}

impl Default for TrayCategory {
    fn default() -> Self {
        Self::ApplicationStatus
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayIcon {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TrayItemAdded {
    pub item: TrayItem,
}

#[derive(Debug, Clone)]
pub struct TrayItemRemoved {
    pub service: String,
}

#[derive(Debug, Clone)]
pub struct TrayItemUpdated {
    pub service: String,
}

#[derive(Debug, Clone)]
pub struct TrayStateChanged;
