#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplicationInfo {
    pub name: String,
    pub name_lower: String, // Pour la recherche rapide
    pub exec: String,
    pub icon: Option<String>,
    pub icon_path: Option<String>,
}
