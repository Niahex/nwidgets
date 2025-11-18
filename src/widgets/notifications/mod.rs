mod notifications_widget;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::Connection;
use tokio::sync::mpsc;

pub use notifications_widget::NotificationsWidget;

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub urgency: u8,
    pub timestamp: u64,
}

pub struct NotificationService {
    notifications: Arc<Mutex<Vec<Notification>>>,
    sender: mpsc::UnboundedSender<Notification>,
}

struct NotificationServer {
    notifications: Arc<Mutex<Vec<Notification>>>,
    sender: mpsc::UnboundedSender<Notification>,
    next_id: u32,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
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
        println!("[NOTIF] 📨 Received notification - app: '{}', summary: '{}', body: '{}'",
            app_name, summary, body);

        let id = if replaces_id > 0 {
            println!("[NOTIF] Replacing notification ID: {}", replaces_id);
            replaces_id
        } else {
            self.next_id += 1;
            println!("[NOTIF] New notification ID: {}", self.next_id);
            self.next_id
        };

        let urgency = if let Some(value) = hints.get("urgency") {
            if let Ok(u) = value.downcast_ref::<u8>() {
                println!("[NOTIF] Urgency from hints: {}", u);
                u.clone()
            } else {
                println!("[NOTIF] Failed to parse urgency, using default: 1");
                1
            }
        } else {
            println!("[NOTIF] No urgency hint, using default: 1");
            1
        };

        let notification = Notification {
            id,
            app_name: app_name.clone(),
            summary: summary.clone(),
            body: body.clone(),
            urgency,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        {
            let mut notifications = self.notifications.lock().unwrap();
            if let Some(pos) = notifications.iter().position(|n| n.id == id) {
                println!("[NOTIF] Updating existing notification at position {}", pos);
                notifications[pos] = notification.clone();
            } else {
                println!("[NOTIF] Adding new notification");
                notifications.push(notification.clone());
            }
            notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            notifications.truncate(10);
            println!("[NOTIF] Total notifications in storage: {}", notifications.len());
        }

        match self.sender.send(notification.clone()) {
            Ok(_) => println!("[NOTIF] ✅ Notification sent to channel successfully"),
            Err(e) => println!("[NOTIF] ❌ Failed to send notification to channel: {}", e),
        }

        id
    }

    fn close_notification(&mut self, id: u32) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.retain(|n| n.id != id);
    }

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
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Notification>) {
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let (sender, receiver) = mpsc::unbounded_channel();

        let service = Self {
            notifications: notifications.clone(),
            sender: sender.clone(),
        };

        (service, receiver)
    }

    pub fn start_dbus_server(&self) {
        let notifications = self.notifications.clone();
        let sender = self.sender.clone();

        println!("[NOTIF] 🚀 Starting D-Bus server thread");

        std::thread::spawn(move || {
            println!("[NOTIF] 🔧 D-Bus thread started, creating runtime");
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                println!("[NOTIF] 🔧 Running D-Bus server");
                if let Err(e) = Self::run_dbus_server(notifications, sender).await {
                    eprintln!("[NOTIF] ❌ Erreur D-Bus: {}", e);
                } else {
                    println!("[NOTIF] ✅ D-Bus server running");
                }
            });
        });
    }

    async fn run_dbus_server(
        notifications: Arc<Mutex<Vec<Notification>>>,
        sender: mpsc::UnboundedSender<Notification>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("[NOTIF] 🔌 Connecting to D-Bus session bus");
        let connection = Connection::session().await?;
        println!("[NOTIF] ✅ Connected to D-Bus session bus");

        let server = NotificationServer {
            notifications,
            sender,
            next_id: 0,
        };

        println!("[NOTIF] 📍 Registering object at /org/freedesktop/Notifications");
        connection
            .object_server()
            .at("/org/freedesktop/Notifications", server)
            .await?;
        println!("[NOTIF] ✅ Object registered");

        println!("[NOTIF] 🏷️  Requesting name org.freedesktop.Notifications");
        connection
            .request_name("org.freedesktop.Notifications")
            .await?;
        println!("[NOTIF] ✅ Name acquired: org.freedesktop.Notifications");
        println!("[NOTIF] 🎉 D-Bus server is now ready to receive notifications!");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}
