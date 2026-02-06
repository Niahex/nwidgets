use std::path::PathBuf;

use crate::widgets::chat::types::DEFAULT_URL;

fn state_file() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("nwidgets")
        .join("chat_url")
}

pub fn load_url() -> String {
    std::fs::read_to_string(state_file()).unwrap_or_else(|_| DEFAULT_URL.to_string())
}

pub fn save_url(url: &str) {
    if let Some(parent) = state_file().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(state_file(), url);
}
