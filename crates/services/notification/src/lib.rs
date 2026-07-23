use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global, SharedString};
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::Connection;

pub const HISTORY_CAPACITY: usize = 50;

#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: u32,
    pub app_name: SharedString,
    pub summary: SharedString,
    pub body: SharedString,
    pub urgency: u8,
    pub timestamp: u64,
    pub actions: Vec<String>,
    pub app_icon: SharedString,
}

#[derive(Clone)]
pub struct NotificationAdded {
    pub notification: Notification,
}

#[derive(Clone)]
pub struct NotificationsCleared;

struct DbusState {
    next_id: u32,
    sender: Option<tokio::sync::mpsc::UnboundedSender<Notification>>,
}

struct NotificationServer {
    state: Arc<Mutex<DbusState>>,
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
        let mut st = self.state.lock();

        if app_name.to_lowercase() == "spotify" {
            let id = if replaces_id > 0 { replaces_id } else { st.next_id += 1; st.next_id };
            return id;
        }

        let id = if replaces_id > 0 { replaces_id } else { st.next_id += 1; st.next_id };

        let urgency = hints
            .get("urgency")
            .and_then(|v| v.downcast_ref::<u8>().ok())
            .unwrap_or(1);

        let notification = Notification {
            id,
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

        if let Some(ref sender) = st.sender {
            let _ = sender.send(notification);
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

pub struct NotificationService {
    pub history: VecDeque<Notification>,
}

impl EventEmitter<NotificationAdded> for NotificationService {}
impl EventEmitter<NotificationsCleared> for NotificationService {}

struct GlobalNotificationService(Entity<NotificationService>);
impl Global for GlobalNotificationService {}

impl NotificationService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalNotificationService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|_| Self {
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
        });
        cx.set_global(GlobalNotificationService(service.clone()));

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Notification>();
        let dbus_state = Arc::new(Mutex::new(DbusState {
            next_id: 0,
            sender: Some(tx),
        }));

        let state_for_dbus = Arc::clone(&dbus_state);

        // Start D-Bus server
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Ok(connection) = Connection::session().await {
                let server = NotificationServer { state: state_for_dbus };
                if connection
                    .object_server()
                    .at("/org/freedesktop/Notifications", server)
                    .await
                    .is_ok()
                {
                    loop {
                        if connection.request_name("org.freedesktop.Notifications").await.is_ok() {
                            log::info!("Successfully registered D-Bus name org.freedesktop.Notifications");
                            std::future::pending::<()>().await;
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        })
        .detach();

        // UI Listener
        let weak = service.downgrade();
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(notification) = rx.recv().await {
                    let _ = weak.update(&mut cx, |this, cx| {
                        this.history.push_front(notification.clone());
                        if this.history.len() > HISTORY_CAPACITY {
                            this.history.pop_back();
                        }
                        cx.emit(NotificationAdded { notification });
                        cx.notify();
                    });
                }
            }
        })
        .detach();

        service
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.history.clear();
        cx.emit(NotificationsCleared);
        cx.notify();
    }
}
