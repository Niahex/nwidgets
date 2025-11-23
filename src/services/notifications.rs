use glib::MainContext;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::Connection;

#[derive(Debug, Clone)]
pub struct Notification {
    #[allow(dead_code)]
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub urgency: u8,
    pub timestamp: u64,
}

pub struct NotificationService;

static NOTIFICATION_SENDER: Mutex<Option<mpsc::Sender<Notification>>> = Mutex::new(None);

struct NotificationServer {
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
        println!(
            "[NOTIF] 📨 Received notification - app: '{}', summary: '{}', body: '{}'",
            app_name, summary, body
        );

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

        // Envoyer via le sender global
        if let Ok(sender_guard) = NOTIFICATION_SENDER.lock() {
            if let Some(sender) = sender_guard.as_ref() {
                match sender.send(notification.clone()) {
                    Ok(_) => println!("[NOTIF] ✅ Notification sent to channel successfully"),
                    Err(e) => println!("[NOTIF] ❌ Failed to send notification to channel: {}", e),
                }
            }
        }

        id
    }

    fn close_notification(&mut self, _id: u32) {
        // Optionnel : on pourrait gérer la fermeture explicite ici
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
    fn start_dbus_server_once() {
        static INIT: std::sync::Once = std::sync::Once::new();

        INIT.call_once(|| {
            println!("[NOTIF] 🚀 Starting D-Bus server thread");

            std::thread::spawn(move || {
                println!("[NOTIF] 🔧 D-Bus thread started, creating runtime");
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    println!("[NOTIF] 🔧 Running D-Bus server");
                    if let Err(e) = Self::run_dbus_server().await {
                        eprintln!("[NOTIF] ❌ Erreur D-Bus: {}", e);
                    } else {
                        println!("[NOTIF] ✅ D-Bus server running");
                    }
                });
            });
        });
    }

    async fn run_dbus_server() -> Result<(), Box<dyn std::error::Error>> {
        println!("[NOTIF] 🔌 Connecting to D-Bus session bus");
        let connection = Connection::session().await?;
        println!("[NOTIF] ✅ Connected to D-Bus session bus");

        let server = NotificationServer { next_id: 0 };

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

    /// S'abonner aux notifications
    /// Le callback sera appelé pour chaque nouvelle notification reçue
    pub fn subscribe_notifications<F>(callback: F)
    where
        F: Fn(Notification) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        // Initialiser le sender global
        {
            let mut sender_guard = NOTIFICATION_SENDER.lock().unwrap();
            *sender_guard = Some(tx);
        }

        // Démarrer le serveur D-Bus (une seule fois)
        Self::start_dbus_server_once();

        // Créer le bridge vers glib
        let (async_tx, async_rx) = async_channel::unbounded();

        std::thread::spawn(move || {
            while let Ok(notification) = rx.recv() {
                if async_tx.send_blocking(notification).is_err() {
                    break;
                }
            }
        });

        MainContext::default().spawn_local(async move {
            while let Ok(notification) = async_rx.recv().await {
                callback(notification);
            }
        });
    }
}
