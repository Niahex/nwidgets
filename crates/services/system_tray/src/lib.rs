use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global};
use zbus::{connection::Builder, interface};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayItem {
    pub id: String,
    pub title: String,
    pub icon_name: String,
    pub tooltip: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SystemTrayState {
    pub items: Vec<TrayItem>,
}

#[derive(Debug, Clone)]
pub struct SystemTrayStateChanged;

pub struct SystemTrayService {
    pub state: SystemTrayState,
}

impl EventEmitter<SystemTrayStateChanged> for SystemTrayService {}

struct GlobalSystemTrayService(Entity<SystemTrayService>);
impl Global for GlobalSystemTrayService {}

struct StatusNotifierWatcher {
    tx: mpsc::UnboundedSender<String>,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    async fn register_status_notifier_item(&self, service: String) {
        let _ = self.tx.unbounded_send(service);
    }

    async fn register_status_notifier_host(&self, _service: String) {}

    #[zbus(property)]
    async fn registered_status_notifier_items(&self) -> Vec<String> {
        vec![]
    }

    #[zbus(property)]
    async fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    async fn protocol_version(&self) -> i32 {
        0
    }
}

impl SystemTrayService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalSystemTrayService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        // Initial default system tray items
        let initial_items = vec![
            TrayItem {
                id: "nwidgets".to_string(),
                title: "nwidgets".to_string(),
                icon_name: "widgets".to_string(),
                tooltip: "nwidgets desktop".to_string(),
                category: "ApplicationStatus".to_string(),
            },
            TrayItem {
                id: "nbuilder".to_string(),
                title: "nbuilder".to_string(),
                icon_name: "architecture".to_string(),
                tooltip: "nbuilder pipeline manager".to_string(),
                category: "ApplicationStatus".to_string(),
            },
            TrayItem {
                id: "security".to_string(),
                title: "Sécurité".to_string(),
                icon_name: "shield".to_string(),
                tooltip: "Sécurité système active".to_string(),
                category: "Hardware".to_string(),
            },
        ];

        let service = cx.new(|_cx| Self {
            state: SystemTrayState {
                items: initial_items,
            },
        });

        cx.set_global(GlobalSystemTrayService(service.clone()));

        let (tx, mut rx) = mpsc::unbounded::<String>();

        // Register D-Bus StatusNotifierWatcher
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Ok(builder) = Builder::session() {
                if let Ok(builder) = builder.name("org.kde.StatusNotifierWatcher") {
                    if let Ok(builder) = builder.serve_at(
                        "/StatusNotifierWatcher",
                        StatusNotifierWatcher { tx },
                    ) {
                        if let Ok(_conn) = builder.build().await {
                            std::future::pending::<()>().await;
                        }
                    }
                }
            }
        })
        .detach();

        // UI Thread listener
        let service_entity = service.clone();
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(item_path) = rx.next().await {
                    let _ = cx.update(|cx| {
                        service_entity.update(cx, |srv, cx| {
                            let item_name = item_path
                                .trim_start_matches('/')
                                .trim_start_matches(':')
                                .to_string();
                            if !srv.state.items.iter().any(|i| i.id == item_name) {
                                srv.state.items.push(TrayItem {
                                    id: item_name.clone(),
                                    title: item_name.clone(),
                                    icon_name: "tune".to_string(),
                                    tooltip: item_name,
                                    category: "ApplicationStatus".to_string(),
                                });
                                cx.emit(SystemTrayStateChanged);
                                cx.notify();
                            }
                        });
                    });
                }
            }
        })
        .detach();

        service
    }
}
