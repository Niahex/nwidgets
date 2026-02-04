use std::sync::Arc;
use parking_lot::RwLock;

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug)]
pub struct ClipboardEntry {
    pub id: u64,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct ClipboardService {
    state: Arc<RwLock<ClipboardState>>,
    last_content: Arc<RwLock<String>>,
}

#[derive(Default)]
struct ClipboardState {
    history: Vec<ClipboardEntry>,
    max_entries: usize,
}

impl ClipboardService {
    pub fn new() -> Self {
        let mut initial_state = ClipboardState::default();
        initial_state.max_entries = 50;

        let state = Arc::new(RwLock::new(initial_state));
        let last_content = Arc::new(RwLock::new(String::new()));
        
        let state_clone = state.clone();
        let last_content_clone = last_content.clone();
        
        TOKIO_RUNTIME.spawn(async move {
            Self::monitor_clipboard(state_clone, last_content_clone).await;
        });

        Self {
            state,
            last_content,
        }
    }
    
    async fn monitor_clipboard(state: Arc<RwLock<ClipboardState>>, last_content: Arc<RwLock<String>>) {
        loop {
            if let Ok(output) = tokio::process::Command::new("wl-paste")
                .arg("-n")
                .output()
                .await
            {
                if let Ok(content) = String::from_utf8(output.stdout) {
                    let content = content.trim().to_string();
                    
                    if !content.is_empty() {
                        let mut last = last_content.write();
                        if *last != content {
                            ::log::info!("Clipboard changed: {} bytes", content.len());
                            *last = content.clone();
                            
                            let mut state_guard = state.write();
                            let max_entries = state_guard.max_entries;
                            
                            let entry = ClipboardEntry {
                                id: chrono::Utc::now().timestamp_millis() as u64,
                                content,
                                timestamp: chrono::Utc::now().timestamp(),
                            };
                            
                            state_guard.history.insert(0, entry);
                            
                            if state_guard.history.len() > max_entries {
                                state_guard.history.truncate(max_entries);
                            }
                        }
                    }
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    pub fn get_history(&self) -> Vec<ClipboardEntry> {
        self.state.read().history.clone()
    }
    
    pub fn get_last_content(&self) -> String {
        self.last_content.read().clone()
    }

    pub fn add_entry(&self, content: String) {
        let mut state = self.state.write();
        let max_entries = state.max_entries;

        let entry = ClipboardEntry {
            id: chrono::Utc::now().timestamp_millis() as u64,
            content,
            timestamp: chrono::Utc::now().timestamp(),
        };

        state.history.insert(0, entry);

        if state.history.len() > max_entries {
            state.history.truncate(max_entries);
        }
    }

    pub fn clear(&self) {
        self.state.write().history.clear();
    }
}
