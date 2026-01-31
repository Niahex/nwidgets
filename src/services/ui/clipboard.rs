use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Clone, Debug)]
pub struct ClipboardEntry {
    pub id: u64,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct ClipboardService {
    state: Arc<RwLock<ClipboardState>>,
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

        Self {
            state: Arc::new(RwLock::new(initial_state)),
        }
    }

    pub fn get_history(&self) -> Vec<ClipboardEntry> {
        self.state.read().history.clone()
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
