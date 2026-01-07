use gpui::prelude::*;
use gpui::{App, Context, Entity, EventEmitter, Global, SharedString};
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::Connection;

#[derive(Clone, Debug)]
pub struct Notification {
    pub app_name: SharedString,
    pub summary: SharedString,
    pub body: SharedString,
    pub urgency: u8,
    pub timestamp: u64,
    #[allow(dead_code)]
    pub actions: Vec<String>,
    pub app_icon: SharedString,
}

#[derive(Clone)]
pub struct NotificationAdded {
    pub notification: Notification,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct NotificationsEmpty;

// État interne partagé protégé par un Mutex
struct NotificationState {
    sender: Option<tokio::sync::mpsc::UnboundedSender<Notification>>,
    history: VecDeque<Notification>,
}

impl NotificationState {
    fn new() -> Self {
        Self {
            sender: None,
            history: VecDeque::with_capacity(50),
        }
    }
}

// Instance globale unique
static STATE: once_cell::sync::Lazy<Arc<Mutex<NotificationState>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(NotificationState::new())));

pub struct NotificationService {
    pub notifications: Arc<parking_lot::RwLock<Vec<Notification>>>,
}

impl EventEmitter<NotificationAdded> for NotificationService {}
impl EventEmitter<NotificationsEmpty> for NotificationService {}

struct NotificationServer {
    next_id: u32,
    state: Arc<Mutex<NotificationState>>,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    #[allow(clippy::too_many_arguments)]
    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
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

        // Extraction de l'urgence (default: 1/Normal)
        // 0: Low, 1: Normal, 2: Critical
        let urgency = hints
            .get("urgency")
            .and_then(|v| v.downcast_ref::<u8>().ok())
            .unwrap_or(1);

        let notification = Notification {
            app_name: app_name.into(),
            summary: summary.into(),
            body: body.into(),
            urgency,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            actions,
            app_icon: app_icon.into(),
        };

        // Mise à jour de l'historique et envoi
        let mut state = self.state.lock();

        // 1. Mise à jour de l'historique
        state.history.push_front(notification.clone());
        if state.history.len() > 50 {
            state.history.pop_back();
        }

        // 2. Envoi via le channel tokio
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
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Démarrer le serveur D-Bus une seule fois
        Self::start_dbus_server_once();

        let notifications = Arc::new(parking_lot::RwLock::new(Vec::new()));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // Initialiser le sender dans l'état global
        {
            let mut state = STATE.lock();
            state.sender = Some(tx);
        }

        let notifications_clone = Arc::clone(&notifications);

        // Traitement réactif des notifications (sans polling !)
        cx.spawn(async move |this, cx| {
            while let Some(notification) = rx.recv().await {
                println!(
                    "[NOTIF_GPUI] 📢 Processing notification: {} - {}",
                    notification.summary, notification.body
                );
                
                notifications_clone.write().push(notification.clone());
                
                let _ = this.update(cx, |_this, cx| {
                    cx.emit(NotificationAdded {
                        notification,
                    });
                    cx.notify();
                });
            }
        })
        .detach();

        Self { notifications }
    }

    fn start_dbus_server_once() {
        static INIT: std::sync::Once = std::sync::Once::new();

        INIT.call_once(|| {
            println!("[NOTIF] 🚀 Starting D-Bus server thread");

            let state_ref = Arc::clone(&STATE);

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime for notifications");
                
                rt.block_on(async {
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

        let server = NotificationServer { next_id: 0, state };

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

    pub fn get_all(&self) -> Vec<Notification> {
        self.notifications.read().clone()
    }

    #[allow(dead_code)]
    pub fn clear(&self, _cx: &mut Context<Self>) {
        self.notifications.write().clear();
    }
}

// Global accessor
struct GlobalNotificationService(Entity<NotificationService>);
impl Global for GlobalNotificationService {}

impl NotificationService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalNotificationService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalNotificationService(service.clone()));
        service
    }
}
