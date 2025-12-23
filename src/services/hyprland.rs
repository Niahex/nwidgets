use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ActiveWindow {
    pub class: String,
    pub title: String,
    #[serde(default)]
    pub initial_class: String,
    #[serde(default)]
    pub initial_title: String,
    #[serde(default)]
    pub address: String,
}

#[derive(Clone)]
pub struct WorkspaceChanged {
    pub workspaces: Vec<Workspace>,
    pub active_workspace_id: i32,
}

#[derive(Clone)]
pub struct ActiveWindowChanged {
    pub window: Option<ActiveWindow>,
}

pub struct HyprlandService {
    workspaces: Arc<RwLock<Vec<Workspace>>>,
    active_workspace_id: Arc<RwLock<i32>>,
    active_window: Arc<RwLock<Option<ActiveWindow>>>,
}

impl EventEmitter<WorkspaceChanged> for HyprlandService {}
impl EventEmitter<ActiveWindowChanged> for HyprlandService {}

impl HyprlandService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let workspaces = Arc::new(RwLock::new(Vec::new()));
        let active_workspace_id = Arc::new(RwLock::new(1));
        let active_window = Arc::new(RwLock::new(None));

        // Fetch initial state
        let (initial_workspaces, initial_active) = Self::fetch_hyprland_data();
        let initial_window = Self::fetch_active_window();

        *workspaces.write() = initial_workspaces.clone();
        *active_workspace_id.write() = initial_active;
        *active_window.write() = initial_window.clone();

        // Spawn background task to monitor Hyprland events
        let workspaces_clone = Arc::clone(&workspaces);
        let active_workspace_id_clone = Arc::clone(&active_workspace_id);
        let active_window_clone = Arc::clone(&active_window);

        cx.spawn(async move |this, cx| {
            Self::monitor_hyprland_events(
                this,
                workspaces_clone,
                active_workspace_id_clone,
                active_window_clone,
                cx,
            )
            .await
        })
        .detach();

        Self {
            workspaces,
            active_workspace_id,
            active_window,
        }
    }

    pub fn workspaces(&self) -> Vec<Workspace> {
        self.workspaces.read().clone()
    }

    pub fn active_workspace_id(&self) -> i32 {
        *self.active_workspace_id.read()
    }

    pub fn active_window(&self) -> Option<ActiveWindow> {
        self.active_window.read().clone()
    }

    pub fn switch_to_workspace(&self, workspace_id: i32) {
        std::thread::spawn(move || {
            let _ = Self::hyprctl(&["dispatch", "workspace", &workspace_id.to_string()]);
        });
    }

    async fn monitor_hyprland_events(
        this: WeakEntity<Self>,
        workspaces: Arc<RwLock<Vec<Workspace>>>,
        active_workspace_id: Arc<RwLock<i32>>,
        active_window: Arc<RwLock<Option<ActiveWindow>>>,
        cx: &mut AsyncApp,
    ) {
        let hypr_sig = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(sig) => sig,
            Err(_) => {
                return;
            }
        };

        let socket_path = format!(
            "/run/user/{}/hypr/{}/.socket2.sock",
            std::env::var("UID").unwrap_or_else(|_| "1000".to_string()),
            hypr_sig
        );

        // Run blocking socket IO on background executor
        let (tx, mut rx) = futures::channel::mpsc::unbounded();

        cx.background_executor()
            .spawn(async move {
                if let Ok(stream) = UnixStream::connect(&socket_path) {
                    let reader = BufReader::new(stream);
                    for line in reader.lines().map_while(Result::ok) {
                        if line.starts_with("workspace>>")
                            || line.starts_with("createworkspace>>")
                            || line.starts_with("destroyworkspace>>")
                            || line.starts_with("activewindow>>")
                            || line.starts_with("closewindow>>")
                            || line.starts_with("openwindow>>")
                        {
                            let _ = tx.unbounded_send(());
                        }
                    }
                }
            })
            .detach();

        // Process events on foreground
        while rx.next().await.is_some() {
            let (new_workspaces, new_active_id) = Self::fetch_hyprland_data();
            let new_window = Self::fetch_active_window();

            let workspace_changed = {
                let mut ws = workspaces.write();
                let mut active_id = active_workspace_id.write();
                let changed = *ws != new_workspaces || *active_id != new_active_id;
                if changed {
                    *ws = new_workspaces.clone();
                    *active_id = new_active_id;
                }
                changed
            };

            let window_changed = {
                let mut win = active_window.write();
                let changed = *win != new_window;
                if changed {
                    *win = new_window.clone();
                }
                changed
            };

            if workspace_changed || window_changed {
                if let Ok(()) = this.update(cx, |_this, cx| {
                    if workspace_changed {
                        cx.emit(WorkspaceChanged {
                            workspaces: new_workspaces.clone(),
                            active_workspace_id: new_active_id,
                        });
                    }
                    if window_changed {
                        cx.emit(ActiveWindowChanged {
                            window: new_window.clone(),
                        });
                    }
                    cx.notify();
                }) {}
            }
        }
    }

    fn fetch_hyprland_data() -> (Vec<Workspace>, i32) {
        let workspaces_json = Self::hyprctl(&["workspaces", "-j"]);
        let active_workspace_json = Self::hyprctl(&["activeworkspace", "-j"]);

        let workspaces: Vec<Workspace> = serde_json::from_str(&workspaces_json).unwrap_or_default();

        let active_workspace_id: i32 =
            serde_json::from_str::<serde_json::Value>(&active_workspace_json)
                .map(|v| v["id"].as_i64().unwrap_or(1) as i32)
                .unwrap_or(1);

        (workspaces, active_workspace_id)
    }

    fn fetch_active_window() -> Option<ActiveWindow> {
        let active_window_json = Self::hyprctl(&["activewindow", "-j"]);

        if active_window_json.trim().is_empty() || active_window_json == "{}" {
            return None;
        }

        serde_json::from_str::<ActiveWindow>(&active_window_json).ok()
    }

    fn hyprctl(args: &[&str]) -> String {
        std::process::Command::new("hyprctl")
            .args(args)
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .unwrap_or_default()
    }
}

// Global accessor
struct GlobalHyprlandService(Entity<HyprlandService>);
impl Global for GlobalHyprlandService {}

impl HyprlandService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalHyprlandService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalHyprlandService(service.clone()));
        service
    }
}
