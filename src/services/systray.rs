use zbus::{Connection, proxy, interface, SignalContext, MessageHeader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use futures_util::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayItem {
    pub id: String,
    pub title: String,
    pub icon_name: String,
    pub service: String,
    pub object_path: String,
}

// Interface Proxy pour l'item
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
}

// Implémentation du Watcher (Serveur D-Bus)
struct StatusNotifierWatcherImpl {
    registered_items: Arc<Mutex<Vec<String>>>,
    // On garde une référence vers le sender pour notifier le thread UI directement lors de l'enregistrement
    update_sender: mpsc::Sender<()>,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcherImpl {
    fn register_status_notifier_item(&mut self, #[zbus(header)] hdr: MessageHeader<'_>, service: &str) -> zbus::fdo::Result<()> {
        let sender = hdr.sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".into()))?;

        let service_str = if service.starts_with('/') {
            format!("{}{}", sender, service)
        } else {
            service.to_string()
        };

        println!("[SYSTRAY] Registering item: {}", service_str);

        let mut items = self.registered_items.lock().unwrap();
        if !items.contains(&service_str) {
            items.push(service_str.clone());
            // Signaler qu'un nouvel item est arrivé pour déclencher une mise à jour
            let _ = self.update_sender.send(());
        }

        // Emettre le signal DBus standard
        // Note: Dans une implémentation complète, on devrait émettre le signal ici via le contexte
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

pub struct SystemTrayService;

impl SystemTrayService {
    pub fn subscribe_systray<F>(callback: F)
    where
        F: Fn(Vec<TrayItem>) + 'static,
    {
        let (tx, rx) = mpsc::channel();
        // Canal interne pour déclencher le rafraîchissement
        let (update_tx, update_rx) = mpsc::channel();

        std::thread::spawn(move || {
            crate::utils::runtime::block_on(async {
                let connection = match Connection::session().await {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[SYSTRAY] Failed to connect to session bus: {}", e);
                        return;
                    }
                };

                let registered_items = Arc::new(Mutex::new(Vec::new()));

                // Créer et enregistrer le watcher
                let watcher = StatusNotifierWatcherImpl {
                    registered_items: registered_items.clone(),
                    update_sender: update_tx.clone(),
                };

                if let Err(e) = connection.object_server().at("/StatusNotifierWatcher", watcher).await {
                    eprintln!("[SYSTRAY] Failed to serve object: {}", e);
                    return;
                }

                if let Err(e) = connection.request_name("org.kde.StatusNotifierWatcher").await {
                    eprintln!("[SYSTRAY] Failed to request name: {}", e);
                    return;
                }

                // Emettre le signal HostRegistered pour dire aux applis qu'on est là
                if let Ok(iface_ref) = connection.object_server().interface::<_, StatusNotifierWatcherImpl>("/StatusNotifierWatcher").await {
                    let _ = StatusNotifierWatcherImpl::status_notifier_host_registered(iface_ref.signal_context()).await;
                }

                // Boucle de gestion des mises à jour
                // On utilise un select! pour gérer à la fois les enregistrements internes et le monitoring DBus
                // TODO: Ajouter un monitoring NameOwnerChanged pour nettoyer les items disparus (crash)

                loop {
                    // Attendre un signal de mise à jour (nouvel item enregistré)
                    if update_rx.recv().is_ok() {
                        // Petit délai pour laisser le temps à l'application de s'initialiser sur DBus
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                        let current_items = registered_items.lock().unwrap().clone();
                        let mut tray_items = Vec::new();

                        for service_str in current_items {
                            if let Some(item) = Self::query_item(&connection, &service_str).await {
                                tray_items.push(item);
                            }
                        }

                        if tx.send(tray_items).is_err() {
                            break;
                        }
                    }
                }
            });
        });

        crate::utils::subscription::ServiceSubscription::subscribe(rx, callback);
    }

    async fn query_item(connection: &Connection, service_str: &str) -> Option<TrayItem> {
        // Format attendu: ":1.XX/Object/Path" ou "org.package/Object/Path"
        let (dest, path) = if let Some(idx) = service_str.find('/') {
            (&service_str[..idx], &service_str[idx..])
        } else {
            return None;
        };

        let proxy = StatusNotifierItemProxy::builder(connection)
            .destination(dest).ok()?
            .path(path).ok()?
            .build().await.ok()?;

        let id = proxy.id().await.ok().unwrap_or_default();
        let title = proxy.title().await.unwrap_or_else(|_| id.clone());
        let icon_name = proxy.icon_name().await.unwrap_or_default();

        Some(TrayItem {
            id,
            title,
            icon_name,
            service: dest.to_string(),
            object_path: path.to_string(),
        })
    }
}
