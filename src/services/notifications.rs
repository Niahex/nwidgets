use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
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

        let mut state = self.state.lock();
        state.history.push_front(notification.clone());
        if state.history.len() > 50 {
            state.history.pop_back();
        }

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
        // Démarrer le serveur D-Bus via le runtime global de GPUI
        Self::start_dbus_server(cx);

        let notifications = Arc::new(parking_lot::RwLock::new(Vec::new()));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        {
            let mut state = STATE.lock();
            state.sender = Some(tx);
        }

        let notifications_clone = Arc::clone(&notifications);

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(notification) = rx.recv().await {
                    notifications_clone.write().push(notification.clone());
                    let _ = this.update(&mut cx, |_, cx| {
                        cx.emit(NotificationAdded { notification });
                        cx.notify();
                    });
                }
            }
        })
        .detach();

        Self { notifications }
    }

    fn start_dbus_server(cx: &mut Context<Self>) {
        static INIT: std::sync::Once = std::sync::Once::new();

        INIT.call_once(|| {
            println!("[NOTIF] 🚀 Starting D-Bus server");
            let state_ref = Arc::clone(&STATE);
            
            // On utilise gpui_tokio pour réutiliser le runtime global
            gpui_tokio::Tokio::spawn(cx, async move {
                if let Err(e) = Self::run_dbus_server(state_ref).await {
                    eprintln!("[NOTIF] ❌ D-Bus Error: {e}");
                }
            }).detach();
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
        std::future::pending::<()>().await;
        Ok(())
    }

    pub fn get_all(&self) -> Vec<Notification> {
        self.notifications.read().clone()
    }

    pub fn clear(&self) {
        self.notifications.write().clear();
    }
}

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
