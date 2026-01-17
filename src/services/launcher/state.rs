use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplicationInfo {
    pub name: String,
    pub name_lower: String, // For fast search
    pub exec: String,
    pub icon: Option<String>,
    pub icon_path: Option<String>,
}

// Wrapper pour partager ApplicationInfo sans cloner
#[derive(Debug, Clone)]
pub struct AppRef(pub Arc<ApplicationInfo>);
