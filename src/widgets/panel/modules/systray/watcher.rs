use futures::channel::mpsc::UnboundedSender;
use zbus::{interface, Connection};

pub struct StatusNotifierWatcher {
    tx: UnboundedSender<WatcherEvent>,
    registered_items: parking_lot::RwLock<Vec<String>>,
    registered_hosts: parking_lot::RwLock<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum WatcherEvent {
    ItemRegistered(String),
    ItemUnregistered(String),
    HostRegistered,
}

impl StatusNotifierWatcher {
    pub fn new(tx: UnboundedSender<WatcherEvent>) -> Self {
        Self {
            tx,
            registered_items: parking_lot::RwLock::new(Vec::new()),
            registered_hosts: parking_lot::RwLock::new(Vec::new()),
        }
    }

    pub fn get_registered_items(&self) -> Vec<String> {
        self.registered_items.read().clone()
    }
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    async fn register_status_notifier_item(&mut self, #[zbus(header)] hdr: zbus::message::Header<'_>, service: &str) {
        log::info!("Registering StatusNotifierItem: {}", service);
        
        let service_string = if service.starts_with('/') {
            if let Some(sender) = hdr.sender() {
                format!("{}{}", sender, service)
            } else {
                log::error!("Path-only registration without sender: {}", service);
                return;
            }
        } else {
            service.to_string()
        };

        {
            let mut items = self.registered_items.write();
            if !items.contains(&service_string) {
                items.push(service_string.clone());
            }
        }

        if let Err(e) = self.tx.unbounded_send(WatcherEvent::ItemRegistered(service_string)) {
            log::warn!("Failed to send systray item registered event: {}", e);
        }
    }

    async fn register_status_notifier_host(&mut self, service: &str) {
        log::info!("Registering StatusNotifierHost: {}", service);
        
        let service_owned = service.to_string();
        {
            let mut hosts = self.registered_hosts.write();
            if !hosts.contains(&service_owned) {
                hosts.push(service_owned);
            }
        }

        if let Err(e) = self.tx.unbounded_send(WatcherEvent::HostRegistered) {
            log::warn!("Failed to send systray host registered event: {}", e);
        }
    }

    #[zbus(property)]
    async fn registered_status_notifier_items(&self) -> Vec<String> {
        self.registered_items.read().clone()
    }

    #[zbus(property)]
    async fn is_status_notifier_host_registered(&self) -> bool {
        !self.registered_hosts.read().is_empty()
    }

    #[zbus(property)]
    async fn protocol_version(&self) -> i32 {
        0
    }
}

pub async fn start_watcher(tx: UnboundedSender<WatcherEvent>) -> anyhow::Result<()> {
    let connection = Connection::session().await?;
    
    connection
        .request_name("org.kde.StatusNotifierWatcher")
        .await?;

    let watcher = StatusNotifierWatcher::new(tx);
    
    connection
        .object_server()
        .at("/StatusNotifierWatcher", watcher)
        .await?;

    log::info!("StatusNotifierWatcher D-Bus service started");

    std::future::pending::<()>().await;
    Ok(())
}
