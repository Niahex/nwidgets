use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global};
use std::path::Path;
use zbus::{connection::Builder, interface, Connection, Proxy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayItem {
    pub id: String,
    pub service_path: String,
    pub title: String,
    pub icon_name: String,
    pub icon_path: Option<String>,
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

fn find_icon_path(icon_name: &str, id: &str) -> Option<String> {
    let lower_name = icon_name.to_lowercase();
    let lower_id = id.to_lowercase();

    if lower_name.contains("steam") || lower_id.contains("steam") {
        if let Ok(home) = std::env::var("HOME") {
            let steam_tray = format!("{home}/.local/share/Steam/public/steam_tray_mono.png");
            if Path::new(&steam_tray).exists() {
                return Some(steam_tray);
            }

            let icons_dir = format!("{home}/.local/share/icons/hicolor");
            if let Ok(entries) = std::fs::read_dir(&icons_dir) {
                for entry in entries.flatten() {
                    let apps_dir = entry.path().join("apps");
                    if apps_dir.exists() {
                        if let Ok(app_files) = std::fs::read_dir(&apps_dir) {
                            for app_file in app_files.flatten() {
                                let path_str = app_file.path().to_string_lossy().to_string();
                                if path_str.to_lowercase().contains("steam")
                                    && (path_str.ends_with(".png") || path_str.ends_with(".svg"))
                                {
                                    return Some(path_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !icon_name.is_empty() {
        if let Some(p) = freedesktop_icons::lookup(icon_name).with_size(24).find() {
            return Some(p.to_string_lossy().to_string());
        }
    }

    if !id.is_empty() {
        if let Some(p) = freedesktop_icons::lookup(id).with_size(24).find() {
            return Some(p.to_string_lossy().to_string());
        }
    }

    let candidates = [
        format!("/usr/share/pixmaps/{icon_name}.png"),
        format!("/usr/share/pixmaps/{icon_name}.svg"),
        format!("/usr/share/pixmaps/{id}.png"),
        format!("/usr/share/icons/hicolor/scalable/apps/{icon_name}.svg"),
        format!("/usr/share/icons/hicolor/48x48/apps/{icon_name}.png"),
        format!("/usr/share/icons/hicolor/256x256/apps/{icon_name}.png"),
        format!("/usr/share/icons/hicolor/scalable/apps/{id}.svg"),
        format!("/usr/share/icons/hicolor/48x48/apps/{id}.png"),
    ];

    for path in candidates {
        if Path::new(&path).exists() {
            return Some(path);
        }
    }

    None
}

fn map_tray_icon(icon_name: &str, id: &str) -> String {
    let lower_name = icon_name.to_lowercase();
    let lower_id = id.to_lowercase();

    if lower_name.contains("steam") || lower_id.contains("steam") {
        "sports_esports".to_string()
    } else if lower_name.contains("discord") || lower_id.contains("discord") {
        "forum".to_string()
    } else if lower_name.contains("slack") || lower_id.contains("slack") {
        "work".to_string()
    } else if lower_name.contains("spotify") || lower_id.contains("spotify") {
        "music_note".to_string()
    } else if lower_name.contains("obs") || lower_id.contains("obs") {
        "videocam".to_string()
    } else if lower_name.contains("dropbox") || lower_id.contains("dropbox") {
        "cloud".to_string()
    } else if lower_name.contains("mail") || lower_id.contains("mail") || lower_name.contains("thunderbird") {
        "mail".to_string()
    } else if lower_name.contains("telegram") || lower_id.contains("telegram") || lower_name.contains("signal") {
        "send".to_string()
    } else if !icon_name.is_empty() {
        icon_name.to_string()
    } else {
        "tune".to_string()
    }
}

fn clean_tray_title(raw_title: &str, raw_id: &str, item_path: &str) -> String {
    let candidate = if !raw_title.is_empty() && !raw_title.starts_with('/') {
        raw_title
    } else if !raw_id.is_empty() && !raw_id.starts_with('/') {
        raw_id
    } else {
        item_path.rsplit('/').next().unwrap_or(item_path)
    };

    let clean = candidate.trim_start_matches('/').trim();
    if clean.is_empty() {
        return "Application".to_string();
    }

    let mut chars = clean.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

async fn fetch_tray_item_info(item_path: String) -> (String, String, String, Option<String>, String) {
    let conn = match Connection::session().await {
        Ok(c) => c,
        Err(_) => {
            let clean_t = clean_tray_title("", "", &item_path);
            let icon_path = find_icon_path("", &item_path);
            let icon_name = map_tray_icon("", &item_path);
            return (item_path.clone(), clean_t.clone(), icon_name, icon_path, clean_t);
        }
    };

    let (dest, path) = if let Some(idx) = item_path.find('/') {
        if idx == 0 {
            (item_path.clone(), item_path.clone())
        } else {
            (item_path[..idx].to_string(), item_path[idx..].to_string())
        }
    } else {
        (item_path.clone(), "/StatusNotifierItem".to_string())
    };

    let proxy = Proxy::new(
        &conn,
        dest,
        path,
        "org.kde.StatusNotifierItem",
    ).await;

    if let Ok(p) = proxy {
        let icon_name: String = p.get_property("IconName").await.unwrap_or_default();
        let title: String = p.get_property("Title").await.unwrap_or_default();
        let id: String = p.get_property("Id").await.unwrap_or_default();

        let clean_title = clean_tray_title(&title, &id, &item_path);
        let raw_id = if !id.is_empty() { id } else { item_path.clone() };

        let icon_path = find_icon_path(&icon_name, &raw_id);
        let fallback_icon = map_tray_icon(&icon_name, &raw_id);

        (raw_id, clean_title.clone(), fallback_icon, icon_path, clean_title)
    } else {
        let clean_t = clean_tray_title("", "", &item_path);
        let icon_path = find_icon_path("", &item_path);
        let fallback_icon = map_tray_icon("", &item_path);
        (item_path.clone(), clean_t.clone(), fallback_icon, icon_path, clean_t)
    }
}

impl SystemTrayService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalSystemTrayService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|_cx| Self {
            state: SystemTrayState {
                items: vec![],
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
                    if item_path.contains("blueman") {
                        continue;
                    }
                    let (id, title, icon_name, icon_path, tooltip) = fetch_tray_item_info(item_path.clone()).await;
                    if id.contains("blueman") || title.contains("blueman") {
                        continue;
                    }

                    let service_path = item_path.clone();

                    let _ = cx.update(|cx| {
                        service_entity.update(cx, |srv, cx| {
                            if !srv.state.items.iter().any(|i| i.id == id) {
                                srv.state.items.push(TrayItem {
                                    id,
                                    service_path,
                                    title,
                                    icon_name,
                                    icon_path,
                                    tooltip,
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

    pub fn activate_item(&self, service_path: String, x: i32, y: i32, cx: &App) {
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Ok(conn) = Connection::session().await {
                let (dest, path) = if let Some(idx) = service_path.find('/') {
                    if idx == 0 {
                        (service_path.clone(), service_path.clone())
                    } else {
                        (service_path[..idx].to_string(), service_path[idx..].to_string())
                    }
                } else {
                    (service_path.clone(), "/StatusNotifierItem".to_string())
                };

                if let Ok(proxy) = Proxy::new(&conn, dest, path, "org.kde.StatusNotifierItem").await {
                    let _ = proxy.call_method("Activate", &(x, y)).await;
                }
            }
        })
        .detach();
    }

    pub fn context_menu_item(&self, service_path: String, x: i32, y: i32, cx: &App) {
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Ok(conn) = Connection::session().await {
                let (dest, path) = if let Some(idx) = service_path.find('/') {
                    if idx == 0 {
                        (service_path.clone(), service_path.clone())
                    } else {
                        (service_path[..idx].to_string(), service_path[idx..].to_string())
                    }
                } else {
                    (service_path.clone(), "/StatusNotifierItem".to_string())
                };

                if let Ok(proxy) = Proxy::new(&conn, dest, path, "org.kde.StatusNotifierItem").await {
                    let _ = proxy.call_method("ContextMenu", &(x, y)).await;
                }
            }
        })
        .detach();
    }
}
