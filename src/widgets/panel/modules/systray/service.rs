use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use zbus::{
    proxy,
    zvariant::{OwnedObjectPath, OwnedValue},
    Connection, Result,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SystrayItem {
    pub id: SharedString,
    pub title: SharedString,
    pub icon_name: Option<SharedString>,
    pub service_name: String,
    pub object_path: String,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SystrayState {
    pub items: SmallVec<[SystrayItem; 8]>,
}

#[derive(Clone)]
pub struct SystrayChanged;

pub struct SystrayService {
    state: Arc<RwLock<SystrayState>>,
}

impl EventEmitter<SystrayChanged> for SystrayService {}

type ManagedObjects = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;

// --- DBus Interfaces ---

#[proxy(
    default_service = "org.freedesktop.DBus",
    default_path = "/org/freedesktop/DBus",
    interface = "org.freedesktop.DBus"
)]
trait DBus {
    fn list_names(&self) -> Result<Vec<String>>;
}

#[proxy(interface = "org.kde.StatusNotifierItem")]
trait StatusNotifierItem {
    #[zbus(property)]
    fn id(&self) -> Result<String>;

    #[zbus(property)]
    fn title(&self) -> Result<String>;

    #[zbus(property)]
    fn icon_name(&self) -> Result<String>;

    #[zbus(property)]
    fn icon_theme_path(&self) -> Result<String>;

    fn activate(&self, x: i32, y: i32) -> Result<()>;
}

// --- Service Implementation ---

impl SystrayService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = Arc::new(RwLock::new(SystrayState::default()));
        let state_clone = Arc::clone(&state);

        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<SystrayState>();

        // 1. Start StatusNotifierWatcher D-Bus server
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Err(e) = super::dbus_watcher::run_watcher_server().await {
                log::error!("Failed to start StatusNotifierWatcher: {e}");
            }
        })
        .detach();
        
        // 2. Worker Task (Tokio)

        // 3. UI Task (GPUI)
        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(new_state) = ui_rx.next().await {
                    let state_changed = {
                        let mut current_state = state_clone.write();
                        if *current_state != new_state {
                            *current_state = new_state;
                            true
                        } else {
                            false
                        }
                    };

                    if state_changed {
                        let _ = this.update(&mut cx, |_, cx| {
                            cx.emit(SystrayChanged);
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();

        Self { state }
    }

    pub fn items(&self) -> Vec<SystrayItem> {
        self.state.read().items.to_vec()
    }

    pub fn state(&self) -> SystrayState {
        self.state.read().clone()
    }

    async fn systray_worker(ui_tx: futures::channel::mpsc::UnboundedSender<SystrayState>) {
        let conn = match Connection::session().await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to connect to session bus: {e}");
                return;
            }
        };

        // Initial fetch
        let initial_state = Self::fetch_systray_items(&conn).await;
        let _ = ui_tx.unbounded_send(initial_state);

        // Poll for changes every 5 seconds
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let new_state = Self::fetch_systray_items(&conn).await;
            let _ = ui_tx.unbounded_send(new_state);
        }
    }

    async fn fetch_systray_items(conn: &Connection) -> SystrayState {
        let mut items = SmallVec::new();

        // Get all D-Bus names
        let dbus_proxy = match DBusProxy::new(conn).await {
            Ok(p) => p,
            Err(e) => {
                log::error!("Failed to create DBus proxy: {e}");
                return SystrayState { items };
            }
        };

        let names = match dbus_proxy.list_names().await {
            Ok(n) => n,
            Err(e) => {
                log::error!("Failed to list D-Bus names: {e}");
                return SystrayState { items };
            }
        };

        // Look for StatusNotifierItem services
        for name in names {
            if name.starts_with("org.kde.StatusNotifierItem-") {
                if let Some(item) = Self::fetch_item_info(conn, &name).await {
                    items.push(item);
                }
            }
        }

        SystrayState { items }
    }

    async fn fetch_item_info(conn: &Connection, service_name: &str) -> Option<SystrayItem> {
        // Try common paths
        let paths = vec![
            "/StatusNotifierItem",
            "/org/kde/StatusNotifierItem",
        ];

        for path in paths {
            let proxy = StatusNotifierItemProxy::builder(conn)
                .destination(service_name)
                .ok()?
                .path(path)
                .ok()?
                .build()
                .await
                .ok()?;

            let id = proxy.id().await.ok()?;
            let title = proxy.title().await.unwrap_or_else(|_| id.clone());
            let icon_name = proxy.icon_name().await.ok();

            return Some(SystrayItem {
                id: id.clone().into(),
                title: title.into(),
                icon_name: icon_name.map(|s| s.into()),
                service_name: service_name.to_string(),
                object_path: path.to_string(),
            });
        }

        None
    }

    pub async fn activate_item(service_name: &str, object_path: &str) {
        let conn = match Connection::session().await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to connect to session bus: {e}");
                return;
            }
        };

        let proxy_result = StatusNotifierItemProxy::builder(&conn)
            .destination(service_name)
            .and_then(|b| b.path(object_path));
        
        let builder = match proxy_result {
            Ok(b) => b,
            Err(e) => {
                log::error!("Failed to create proxy builder: {e}");
                return;
            }
        };
        
        let proxy = match builder.build().await {
            Ok(p) => p,
            Err(e) => {
                log::error!("Failed to build proxy: {e}");
                return;
            }
        };

        if let Err(e) = proxy.activate(0, 0).await {
            log::error!("Failed to activate systray item: {e}");
        }
    }
}

// Global accessor
struct GlobalSystrayService(Entity<SystrayService>);
impl Global for GlobalSystrayService {}

impl SystrayService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalSystrayService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalSystrayService(service.clone()));
        service
    }
}
