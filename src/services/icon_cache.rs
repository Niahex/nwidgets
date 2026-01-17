use gpui::SharedString;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct IconCache {
    cache: Arc<RwLock<HashMap<String, SharedString>>>,
}

impl IconCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, path: &str) -> Option<SharedString> {
        self.cache.read().get(path).cloned()
    }

    pub fn insert(&self, path: String, icon: SharedString) {
        self.cache.write().insert(path, icon);
    }

    pub fn get_or_load(&self, path: &str) -> SharedString {
        if let Some(cached) = self.get(path) {
            return cached;
        }

        let icon: SharedString = path.to_string().into();
        self.insert(path.to_string(), icon.clone());
        icon
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}
