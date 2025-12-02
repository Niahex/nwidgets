use once_cell::sync::Lazy;
use std::sync::{mpsc, Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct ChatState {
    pub is_visible: bool,
    pub selected_site_name: String,
    pub selected_site_url: String,
}

type ChatStateSender = mpsc::Sender<ChatState>;

struct ChatStateMonitor {
    subscribers: Arc<Mutex<Vec<ChatStateSender>>>,
    current_state: Arc<Mutex<ChatState>>,
}

impl ChatStateMonitor {
    fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
            current_state: Arc::new(Mutex::new(ChatState::default())),
        }
    }

    fn update_state(&self, state: ChatState) {
        // Mettre à jour l'état actuel
        if let Ok(mut current) = self.current_state.lock() {
            *current = state.clone();
        }

        // Notifier tous les subscribers
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.retain(|tx| tx.send(state.clone()).is_ok());
        }
    }

    fn add_subscriber(&self, tx: ChatStateSender) {
        // Envoyer l'état actuel immédiatement au nouveau subscriber
        if let Ok(state) = self.current_state.lock() {
            let _ = tx.send(state.clone());
        }

        // Ajouter le subscriber à la liste
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(tx);
        }
    }

    fn get_current_state(&self) -> ChatState {
        self.current_state
            .lock()
            .map(|state| state.clone())
            .unwrap_or_default()
    }
}

static MONITOR: Lazy<ChatStateMonitor> = Lazy::new(ChatStateMonitor::new);

pub struct ChatStateService;

impl ChatStateService {
    /// Subscribe to chat state changes
    pub fn subscribe<F>(callback: F)
    where
        F: Fn(ChatState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        MONITOR.add_subscriber(tx);

        crate::utils::subscription::ServiceSubscription::subscribe(rx, callback);
    }

    /// Update visibility only
    pub fn set_visibility(is_visible: bool) {
        let mut state = MONITOR.get_current_state();
        state.is_visible = is_visible;
        MONITOR.update_state(state);
    }

    /// Update selected site
    pub fn set_selected_site(name: String, url: String) {
        let mut state = MONITOR.get_current_state();
        state.selected_site_name = name;
        state.selected_site_url = url;
        MONITOR.update_state(state);
    }
}
