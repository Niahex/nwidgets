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
pub struct WorkspaceChanged;

#[derive(Clone)]
pub struct ActiveWindowChanged;

pub struct HyprlandService {
    workspaces: Arc<RwLock<Vec<Workspace>>>,
    active_workspace_id: Arc<RwLock<i32>>,
    active_window: Arc<RwLock<Option<ActiveWindow>>>,
}

impl EventEmitter<WorkspaceChanged> for HyprlandService {}
impl EventEmitter<ActiveWindowChanged> for HyprlandService {}

impl HyprlandService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (initial_workspaces, initial_active) = Self::fetch_hyprland_data();
        let initial_window = Self::fetch_active_window();

        let workspaces = Arc::new(RwLock::new(initial_workspaces));
        let active_workspace_id = Arc::new(RwLock::new(initial_active));
        let active_window = Arc::new(RwLock::new(initial_window));

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
            Err(_) => return,
        };

        let socket_path = format!(
            "/run/user/{}/hypr/{}/.socket2.sock",
            std::env::var("UID").unwrap_or_else(|_| "1000".to_string()),
            hypr_sig
        );

        let (tx, mut rx) = futures::channel::mpsc::unbounded();

        cx.background_executor()
            .spawn(async move {
                if let Ok(stream) = UnixStream::connect(&socket_path) {
                    let reader = BufReader::new(stream);
                    for line in reader.lines().map_while(Result::ok) {
                        // Notify which type of update is needed
                        if line.starts_with("workspace>>")
                            || line.starts_with("createworkspace>>")
                            || line.starts_with("destroyworkspace>>")
                        {
                            let _ = tx.unbounded_send(true); // Workspace update
                        } else if line.starts_with("activewindow>>")
                            || line.starts_with("closewindow>>")
                            || line.starts_with("openwindow>>")
                        {
                            let _ = tx.unbounded_send(false); // Window update
                        }
                    }
                }
            })
            .detach();

        while let Some(is_workspace_event) = rx.next().await {
            // Drain remaining events in channel to avoid redundant updates
            let mut more_ws = false;
            let mut more_win = false;
            while let Ok(Some(ev)) = rx.try_next() {
                if ev {
                    more_ws = true;
                } else {
                    more_win = true;
                }
            }

            let do_ws = is_workspace_event || more_ws;
            let do_win = !is_workspace_event || more_win;

            let mut updated_ws = None;
            let mut updated_active_id = None;
            let mut updated_win = None;

            if do_ws {
                let (ws, id) = Self::fetch_hyprland_data();
                updated_ws = Some(ws);
                updated_active_id = Some(id);
            }

            if do_win {
                updated_win = Some(Self::fetch_active_window());
            }

            let workspace_changed =
                if let (Some(new_ws), Some(new_id)) = (updated_ws.clone(), updated_active_id) {
                    let mut ws = workspaces.write();
                    let mut active_id = active_workspace_id.write();
                    let changed = *ws != new_ws || *active_id != new_id;
                    if changed {
                        *ws = new_ws;
                        *active_id = new_id;
                    }
                    changed
                } else {
                    false
                };

            let window_changed = if let Some(new_win) = updated_win.clone() {
                let mut win = active_window.write();
                let changed = *win != new_win;
                if changed {
                    *win = new_win;
                }
                changed
            } else {
                false
            };

            if workspace_changed || window_changed {
                let _ = this.update(cx, |_this, cx| {
                    if workspace_changed {
                        cx.emit(WorkspaceChanged);
                    }
                    if window_changed {
                        cx.emit(ActiveWindowChanged);
                    }
                    cx.notify();
                });
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
