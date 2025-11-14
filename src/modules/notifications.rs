use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::Connection;
use tokio::sync::mpsc;

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
        let id = if replaces_id > 0 { replaces_id } else { 
            self.next_id += 1;
            self.next_id
        };

        let urgency = if let Some(value) = hints.get("urgency") {
            if let Ok(u) = value.downcast_ref::<u8>() {
                u.clone()
            } else {
                1
            }
        } else {
            1
        };

        let notification = Notification {
            id,
            app_name,
            summary,
            body,
            urgency,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        {
            let mut notifications = self.notifications.lock().unwrap();
            if let Some(pos) = notifications.iter().position(|n| n.id == id) {
                notifications[pos] = notification.clone();
            } else {
                notifications.push(notification.clone());
            }
            notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            notifications.truncate(10);
        }

        let _ = self.sender.send(notification);
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

        let notifications_clone = notifications.clone();
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::start_dbus_server(notifications_clone, sender_clone).await {
                eprintln!("Erreur D-Bus: {}", e);
            }
        });

        (service, receiver)
    }

    async fn start_dbus_server(
        notifications: Arc<Mutex<Vec<Notification>>>,
        sender: mpsc::UnboundedSender<Notification>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::session().await?;
        
        let server = NotificationServer {
            notifications,
            sender,
            next_id: 0,
        };

        connection
            .object_server()
            .at("/org/freedesktop/Notifications", server)
            .await?;

        connection
            .request_name("org.freedesktop.Notifications")
            .await?;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    pub fn get_notifications(&self) -> Vec<Notification> {
        self.notifications.lock().unwrap().clone()
    }

    pub fn remove_notification(&self, id: u32) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.retain(|n| n.id != id);
    }
}
