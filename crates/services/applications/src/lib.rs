use freedesktop_desktop_entry::{DesktopEntry, Iter};
use freedesktop_icons::lookup;
use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub exec: String,
    pub icon_name: Option<String>,
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApplicationsStateChanged;

pub struct ApplicationsService {
    pub applications: Vec<AppInfo>,
}

impl EventEmitter<ApplicationsStateChanged> for ApplicationsService {}

struct GlobalApplicationsService(Entity<ApplicationsService>);
impl Global for GlobalApplicationsService {}

fn get_cache_path() -> String {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        format!("{runtime_dir}/nlauncher_cache.json")
    } else {
        "/tmp/nlauncher_cache.json".to_string()
    }
}

fn load_from_cache() -> Option<Vec<AppInfo>> {
    let cache_path = get_cache_path();
    let path = Path::new(&cache_path);
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_to_cache(apps: &[AppInfo]) {
    let cache_path = get_cache_path();
    if let Ok(content) = serde_json::to_string(apps) {
        let _ = fs::write(cache_path, content);
    }
}

fn scan_applications() -> Vec<AppInfo> {
    let applications = Arc::new(Mutex::new(Vec::with_capacity(200)));
    let seen_names = Arc::new(Mutex::new(HashSet::new()));

    let paths: Vec<_> = Iter::new(freedesktop_desktop_entry::default_paths()).collect();
    if paths.is_empty() {
        return Vec::new();
    }
    let chunk_size = (paths.len() / 4).max(1);

    let mut handles = Vec::with_capacity(4);

    for chunk in paths.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let applications = Arc::clone(&applications);
        let seen_names = Arc::clone(&seen_names);

        let handle = thread::spawn(move || {
            let mut local_apps = Vec::with_capacity(50);

            for path in chunk {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(desktop_entry) = DesktopEntry::from_str(&path, &content, None::<&[&str]>) {
                        if let Some(name) = desktop_entry.name::<&str>(&[]) {
                            if let Some(exec) = desktop_entry.exec() {
                                {
                                    let Ok(mut seen) = seen_names.lock() else {
                                        continue;
                                    };
                                    if seen.contains(&name.to_string()) {
                                        continue;
                                    }
                                    seen.insert(name.to_string());
                                }

                                let icon_path = desktop_entry
                                    .icon()
                                    .and_then(|icon_name| lookup(icon_name).with_size(24).find())
                                    .map(|p| p.to_string_lossy().to_string());

                                let exec_clean: String = exec
                                    .split_whitespace()
                                    .filter(|part| !part.starts_with('%'))
                                    .collect::<Vec<_>>()
                                    .join(" ");

                                local_apps.push(AppInfo {
                                    name: name.to_string(),
                                    exec: exec_clean,
                                    icon_name: desktop_entry.icon().map(|s| s.to_string()),
                                    icon_path,
                                });
                            }
                        }
                    }
                }
            }

            if let Ok(mut apps) = applications.lock() {
                apps.extend(local_apps);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    let mut apps = match Arc::try_unwrap(applications) {
        Ok(mutex) => mutex.into_inner().unwrap_or_default(),
        Err(arc) => arc.lock().map(|g| g.clone()).unwrap_or_default(),
    };

    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps
}

impl ApplicationsService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalApplicationsService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let cached = load_from_cache().unwrap_or_default();
        let service = cx.new(|_cx| Self {
            applications: cached,
        });

        cx.set_global(GlobalApplicationsService(service.clone()));

        let (tx, mut rx) = mpsc::unbounded::<Vec<AppInfo>>();

        // Async background scanner thread
        gpui_tokio::Tokio::spawn(cx, async move {
            let apps = scan_applications();
            save_to_cache(&apps);
            let _ = tx.unbounded_send(apps);
        })
        .detach();

        // UI Thread listener
        let service_entity = service.clone();
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(new_apps) = rx.next().await {
                    let _ = cx.update(|cx| {
                        service_entity.update(cx, |srv, cx| {
                            if srv.applications != new_apps {
                                srv.applications = new_apps;
                                cx.emit(ApplicationsStateChanged);
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

    pub fn search(&self, query: &str) -> Vec<AppInfo> {
        if query.trim().is_empty() {
            return self.applications.clone();
        }

        let q = query.to_lowercase();
        self.applications
            .iter()
            .filter(|app| app.name.to_lowercase().contains(&q) || app.exec.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }
}
