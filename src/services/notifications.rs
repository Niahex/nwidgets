use once_cell::sync::Lazy;
use parking_lot::Mutex; // Mutex plus rapide et ergonomique
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::Connection;

#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub urgency: u8,
    pub timestamp: u64,
}

// État interne partagé protégé par un Mutex
struct NotificationState {
    sender: Option<mpsc::Sender<Notification>>,
    history: VecDeque<Notification>, // VecDeque est optimisé pour push_front/pop_back
}

impl NotificationState {
    fn new() -> Self {
        Self {
            sender: None,
            history: VecDeque::with_capacity(50),
        }
    }
}

// Instance globale unique lazy
static STATE: Lazy<Arc<Mutex<NotificationState>>> =
    Lazy::new(|| Arc::new(Mutex::new(NotificationState::new())));

pub struct NotificationService;

struct NotificationServer {
    next_id: u32,
    // Le serveur garde une référence vers l'état global
    state: Arc<Mutex<NotificationState>>,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    #[allow(clippy::too_many_arguments)]
    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        _app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        hints: HashMap<String, zbus::zvariant::Value>,
        _expire_timeout: i32,
    ) -> u32 {
        println!("[NOTIF] 📨 Received - app: '{app_name}', summary: '{summary}'");

        // Ignorer les notifications de Spotify
        if app_name.to_lowercase() == "spotify" {
            println!("[NOTIF] 🚫 Ignoring Spotify notification");
            let id = if replaces_id > 0 {
                replaces_id
            } else {
                self.next_id += 1;
                self.next_id
            };
            return id;
        }

        let id = if replaces_id > 0 {
            replaces_id
        } else {
            self.next_id += 1;
            self.next_id
        };

        // Extraction optimisée de l'urgence (default: 1/Normal)
        // 0: Low, 1: Normal, 2: Critical
        let urgency = hints
            .get("urgency")
            .and_then(|v| v.downcast_ref::<u8>().ok())
            .unwrap_or(1);

        let notification = Notification {
            app_name,
            summary,
            body,
            urgency,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Verrouillage unique pour la mise à jour de l'historique ET l'envoi
        let mut state = self.state.lock();

        // 1. Mise à jour de l'historique (O(1) avec VecDeque)
        state.history.push_front(notification.clone());
        if state.history.len() > 50 {
            state.history.pop_back();
        }

        // 2. Envoi via le channel
        if let Some(sender) = &state.sender {
            if let Err(e) = sender.send(notification) {
                eprintln!("[NOTIF] ❌ Failed to send to UI: {e}");
            }
        }

        id
    }

    fn close_notification(&mut self, _id: u32) {}

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "body".to_string(),
            "body-markup".to_string(),
            "actions".to_string(),
            "urgency".to_string(),
        ]
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        (
            "nwidgets".to_string(),
            "nwidgets".to_string(),
            "0.1.0".to_string(),
            "1.2".to_string(),
        )
    }
}

impl NotificationService {
    fn start_dbus_server_once() {
        static INIT: std::sync::Once = std::sync::Once::new();

        INIT.call_once(|| {
            println!("[NOTIF] 🚀 Starting D-Bus server thread");

            // Clonage de l'Arc pour le thread
            let state_ref = Arc::clone(&STATE);

            std::thread::spawn(move || {
                crate::utils::runtime::block_on(async {
                    if let Err(e) = Self::run_dbus_server(state_ref).await {
                        eprintln!("[NOTIF] ❌ D-Bus Error: {e}");
                    }
                });
            });
        });
    }

    async fn run_dbus_server(
        state: Arc<Mutex<NotificationState>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::session().await?;

        let server = NotificationServer {
            next_id: 0,
            state, // Injection de l'état partagé
        };

        connection
            .object_server()
            .at("/org/freedesktop/Notifications", server)
            .await?;

        connection
            .request_name("org.freedesktop.Notifications")
            .await?;

        println!("[NOTIF] ✅ Service ready on org.freedesktop.Notifications");

        // Maintient la connexion active
        std::future::pending::<()>().await;
        Ok(())
    }

    /// Récupérer l'historique des notifications
    /// Retourne un Vec standard pour la compatibilité avec l'interface UI existante
    pub fn get_history() -> Vec<Notification> {
        STATE.lock().history.iter().cloned().collect()
    }

    /// S'abonner aux notifications
    pub fn subscribe_notifications<F>(callback: F)
    where
        F: Fn(Notification) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        // Initialisation du sender
        {
            let mut state = STATE.lock();
            state.sender = Some(tx);
        }

        Self::start_dbus_server_once();

        crate::utils::subscription::ServiceSubscription::subscribe(rx, callback);
    }
}
