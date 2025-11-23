use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use zbus::{Connection, proxy};
use once_cell::sync::Lazy;
use futures::stream::StreamExt;
use glib::{MainContext, Priority};

// Structure pour stocker les informations du menu d'une application
#[derive(Debug, Clone)]
pub struct AppMenuInfo {
    pub window_id: u32,
    pub service_name: String,
    pub object_path: String,
}

// Type pour les callbacks
type AppMenuSender = mpsc::Sender<Option<AppMenuInfo>>;

// Proxy pour l'interface com.canonical.AppMenu.Registrar
#[proxy(
    interface = "com.canonical.AppMenu.Registrar",
    default_service = "com.canonical.AppMenu.Registrar",
    default_path = "/com/canonical/AppMenu/Registrar"
)]
trait AppMenuRegistrar {
    /// Get the menu for a specific window
    fn get_menu_for_window(&self, window_id: u32) -> zbus::Result<(String, String)>;

    /// RegisterWindow signal
    #[zbus(signal)]
    fn window_registered(&self, window_id: u32, service: &str, path: &str) -> zbus::Result<()>;

    /// UnregisterWindow signal
    #[zbus(signal)]
    fn window_unregistered(&self, window_id: u32) -> zbus::Result<()>;
}

static APPMENU_MONITOR: Lazy<AppMenuMonitor> = Lazy::new(|| AppMenuMonitor::new());

/// Structure pour gérer le monitoring centralisé des menus
struct AppMenuMonitor {
    menus: Arc<Mutex<HashMap<u32, AppMenuInfo>>>,
    subscribers: Arc<Mutex<Vec<AppMenuSender>>>,
    started: Arc<Mutex<bool>>,
}

impl AppMenuMonitor {
    fn new() -> Self {
        Self {
            menus: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            started: Arc::new(Mutex::new(false)),
        }
    }

    fn ensure_started(&self) {
        let mut started = self.started.lock().unwrap();
        if *started {
            return;
        }
        *started = true;

        let menus = Arc::clone(&self.menus);
        let subscribers = Arc::clone(&self.subscribers);

        // Utiliser un thread standard au lieu de tokio
        std::thread::spawn(move || {
            // Créer un runtime tokio pour ce thread
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
            match Connection::session().await {
                Ok(connection) => {
                    log::info!("Connected to session bus for AppMenu");

                    match AppMenuRegistrarProxy::new(&connection).await {
                        Ok(proxy) => {
                            log::info!("Connected to AppMenu registrar");

                            // Écouter les signaux d'enregistrement
                            let mut reg_stream = match proxy.receive_window_registered().await {
                                Ok(s) => s,
                                Err(e) => {
                                    log::error!("Failed to subscribe to window_registered: {}", e);
                                    return;
                                }
                            };

                            // Écouter les signaux de désenregistrement
                            let mut unreg_stream = match proxy.receive_window_unregistered().await {
                                Ok(s) => s,
                                Err(e) => {
                                    log::error!("Failed to subscribe to window_unregistered: {}", e);
                                    return;
                                }
                            };

                            loop {
                                tokio::select! {
                                    Some(signal) = reg_stream.next() => {
                                        if let Ok(args) = signal.args() {
                                            let window_id = args.window_id;
                                            let service = args.service.to_string();
                                            let path = args.path.to_string();

                                            log::info!("Window {} registered menu at {}:{}", window_id, service, path);

                                            let menu_info = AppMenuInfo {
                                                window_id,
                                                service_name: service,
                                                object_path: path,
                                            };

                                            // Stocker le menu
                                            if let Ok(mut m) = menus.lock() {
                                                m.insert(window_id, menu_info.clone());
                                            }

                                            // Notifier les abonnés
                                            Self::broadcast_update(&subscribers, Some(menu_info));
                                        }
                                    }
                                    Some(signal) = unreg_stream.next() => {
                                        if let Ok(args) = signal.args() {
                                            let window_id = args.window_id;
                                            log::info!("Window {} unregistered menu", window_id);

                                            // Supprimer le menu
                                            if let Ok(mut m) = menus.lock() {
                                                m.remove(&window_id);
                                            }

                                            // Notifier les abonnés
                                            Self::broadcast_update(&subscribers, None);
                                        }
                                    }
                                    else => break,
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("AppMenu registrar not available: {}", e);
                            log::info!("Global menu support will be disabled");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to connect to session bus: {}", e);
                }
            }
            });
        });
    }

    fn broadcast_update(
        subscribers: &Arc<Mutex<Vec<AppMenuSender>>>,
        menu_info: Option<AppMenuInfo>,
    ) {
        if let Ok(mut subs) = subscribers.lock() {
            subs.retain(|tx| tx.send(menu_info.clone()).is_ok());
        }
    }

    fn subscribe(&self) -> mpsc::Receiver<Option<AppMenuInfo>> {
        let (tx, rx) = mpsc::channel();

        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(tx);
        }

        rx
    }

    fn get_menu_for_window(&self, window_id: u32) -> Option<AppMenuInfo> {
        if let Ok(menus) = self.menus.lock() {
            menus.get(&window_id).cloned()
        } else {
            None
        }
    }
}

pub struct AppMenuService;

impl AppMenuService {
    /// Démarre le monitoring des menus
    pub fn start() {
        APPMENU_MONITOR.ensure_started();
    }

    /// S'abonner aux changements de menu
    /// Retourne un receiver qui reçoit les infos de menu quand elles changent
    pub fn subscribe<F>(callback: F)
    where
        F: Fn(Option<AppMenuInfo>) + Send + 'static,
    {
        APPMENU_MONITOR.ensure_started();

        let rx = APPMENU_MONITOR.subscribe();

        // Envoyer les updates via glib MainContext
        let (tx_glib, rx_glib) = MainContext::channel(Priority::DEFAULT);

        // Thread pour recevoir les updates et les envoyer à glib
        std::thread::spawn(move || {
            while let Ok(menu_info) = rx.recv() {
                let _ = tx_glib.send(menu_info);
            }
        });

        // Recevoir dans le contexte glib et appeler le callback
        rx_glib.attach(None, move |menu_info| {
            callback(menu_info);
            glib::ControlFlow::Continue
        });
    }

    /// Récupère le menu pour une fenêtre spécifique
    pub fn get_menu_for_window(window_id: u32) -> Option<AppMenuInfo> {
        APPMENU_MONITOR.get_menu_for_window(window_id)
    }
}
