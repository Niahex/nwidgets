use zbus::{Connection, proxy, interface, SignalContext, MessageHeader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayItem {
    pub id: String,
    pub title: String,
    pub icon_name: String,
    pub service: String,
    pub object_path: String,
}

// StatusNotifierItem interface proxy
#[proxy(
    interface = "org.kde.StatusNotifierItem",
    default_service = "org.kde.StatusNotifierItem",
    default_path = "/StatusNotifierItem"
)]
trait StatusNotifierItem {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn title(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn icon_theme_path(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn status(&self) -> zbus::Result<String>;

    fn activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn context_menu(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn scroll(&self, delta: i32, orientation: &str) -> zbus::Result<()>;
}

// StatusNotifierWatcher interface proxy
#[proxy(
    interface = "org.kde.StatusNotifierWatcher",
    default_service = "org.kde.StatusNotifierWatcher",
    default_path = "/StatusNotifierWatcher"
)]
trait StatusNotifierWatcher {
    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> zbus::Result<Vec<String>>;

    #[zbus(signal)]
    fn status_notifier_item_registered(&self, service: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    fn status_notifier_item_unregistered(&self, service: &str) -> zbus::Result<()>;

    fn register_status_notifier_item(&self, service: &str) -> zbus::Result<()>;
}

// Our own StatusNotifierWatcher implementation
struct StatusNotifierWatcherImpl {
    registered_items: Arc<Mutex<Vec<String>>>,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcherImpl {
    fn register_status_notifier_item(&mut self, #[zbus(header)] hdr: MessageHeader<'_>, service: &str) -> zbus::fdo::Result<()> {
        let sender = hdr.sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".into()))?;
        let service_str = format!("{}/{}", sender.as_str(), service.trim_start_matches('/'));
        println!("[SYSTRAY_WATCHER] Registering item from {}: {}", sender, service);
        println!("[SYSTRAY_WATCHER] Full service string: {}", service_str);

        let mut items = self.registered_items.lock().unwrap();
        if !items.contains(&service_str) {
            items.push(service_str);
        }
        Ok(())
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.registered_items.lock().unwrap().clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }

    #[zbus(signal)]
    async fn status_notifier_item_registered(signal_ctxt: &SignalContext<'_>, service: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_unregistered(signal_ctxt: &SignalContext<'_>, service: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_registered(signal_ctxt: &SignalContext<'_>) -> zbus::Result<()>;
}

pub struct SystemTrayService {
    items: HashMap<String, TrayItem>,
    registered_items: Arc<Mutex<Vec<String>>>,
}

impl SystemTrayService {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            registered_items: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start monitoring system tray items
    pub async fn start_monitoring(&mut self) -> zbus::Result<Vec<TrayItem>> {
        let connection = Connection::session().await?;

        // Create our own StatusNotifierWatcher
        let watcher = StatusNotifierWatcherImpl {
            registered_items: self.registered_items.clone(),
        };

        // Register the watcher on D-Bus
        connection
            .object_server()
            .at("/StatusNotifierWatcher", watcher)
            .await?;

        // Request the well-known name
        connection
            .request_name("org.kde.StatusNotifierWatcher")
            .await?;

        println!("[SYSTRAY] StatusNotifierWatcher registered successfully");

        // Emit StatusNotifierHostRegistered signal to notify existing applications
        let iface_ref = connection
            .object_server()
            .interface::<_, StatusNotifierWatcherImpl>("/StatusNotifierWatcher")
            .await?;

        StatusNotifierWatcherImpl::status_notifier_host_registered(
            iface_ref.signal_context()
        ).await?;
        println!("[SYSTRAY] Emitted StatusNotifierHostRegistered signal");

        // Give applications a moment to respond to the signal
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Get registered items
        let items = self.registered_items.lock().unwrap().clone();
        println!("[SYSTRAY] Found {} tray items after signal", items.len());

        let mut tray_items = Vec::new();

        for item_service in items {
            if let Some(tray_item) = Self::get_item_info(&connection, &item_service).await {
                println!("[SYSTRAY] Item: {} - {}", tray_item.id, tray_item.title);
                self.items.insert(tray_item.id.clone(), tray_item.clone());
                tray_items.push(tray_item);
            }
        }

        Ok(tray_items)
    }

    async fn get_item_info(connection: &Connection, service: &str) -> Option<TrayItem> {
        // Parse service name - format is "sender_bus_name/object/path"
        // Example: ":1.234/org/ayatana/NotificationItem/steam"
        let (service_name, object_path) = if let Some(slash_pos) = service.find('/') {
            let sender = &service[..slash_pos];
            let path = &service[slash_pos..];
            (sender.to_string(), path.to_string())
        } else {
            // Fallback si pas de slash (ne devrait pas arriver)
            (service.to_string(), "/StatusNotifierItem".to_string())
        };

        println!("[SYSTRAY] Querying item: service={}, path={}", service_name, object_path);

        // Try to connect to the item
        let item_proxy = StatusNotifierItemProxy::builder(connection)
            .destination(service_name.clone())
            .ok()?
            .path(object_path.clone())
            .ok()?
            .build()
            .await
            .ok()?;

        // Get item properties
        let id = item_proxy.id().await.ok()?;
        let title = item_proxy.title().await.unwrap_or_else(|_| id.clone());
        let icon_name = item_proxy.icon_name().await.unwrap_or_default();

        Some(TrayItem {
            id: id.clone(),
            title,
            icon_name,
            service: service_name,
            object_path,
        })
    }

    pub fn get_items(&self) -> Vec<TrayItem> {
        self.items.values().cloned().collect()
    }
}
