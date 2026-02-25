use parking_lot::Mutex;
use std::sync::Arc;
use zbus::Connection;

pub struct StatusNotifierWatcher {
    registered_items: Arc<Mutex<Vec<String>>>,
}

impl StatusNotifierWatcher {
    pub fn new() -> Self {
        Self {
            registered_items: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_registered_items(&self) -> Vec<String> {
        self.registered_items.lock().clone()
    }
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    async fn register_status_notifier_item(&mut self, #[zbus(signal_context)] ctxt: zbus::object_server::SignalEmitter<'_>, service: String) {
        log::info!("StatusNotifierItem registered: {}", service);
        let mut items = self.registered_items.lock();
        if !items.contains(&service) {
            items.push(service.clone());
        }
        drop(items);
        if let Err(e) = Self::status_notifier_item_registered(&ctxt, &service).await {
            log::error!("Failed to emit signal: {e}");
        }
    }

    fn register_status_notifier_host(&self, _service: String) {
        log::info!("StatusNotifierHost registered");
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.registered_items.lock().clone()
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
    async fn status_notifier_item_registered(signal_emitter: &zbus::object_server::SignalEmitter<'_>, service: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_unregistered(signal_emitter: &zbus::object_server::SignalEmitter<'_>, service: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_registered(signal_emitter: &zbus::object_server::SignalEmitter<'_>) -> zbus::Result<()>;
}

pub async fn run_watcher_server() -> zbus::Result<()> {
    let conn = Connection::session().await?;
    
    conn.request_name("org.kde.StatusNotifierWatcher").await?;
    
    let watcher = StatusNotifierWatcher::new();
    conn.object_server()
        .at("/StatusNotifierWatcher", watcher)
        .await?;
    
    log::info!("StatusNotifierWatcher D-Bus server started");
    
    std::future::pending::<()>().await;
    Ok(())
}
