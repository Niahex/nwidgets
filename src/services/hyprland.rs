use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, BackgroundExecutor, Context, Entity, EventEmitter, Global, WeakEntity};
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

#[derive(Clone)]
pub struct FullscreenChanged(pub bool);

pub struct HyprlandService {
    workspaces: Arc<RwLock<Vec<Workspace>>>,
    active_workspace_id: Arc<RwLock<i32>>,
    active_window: Arc<RwLock<Option<ActiveWindow>>>,
    fullscreen_workspace: Arc<RwLock<Option<i32>>>,
    executor: BackgroundExecutor,
}

impl EventEmitter<WorkspaceChanged> for HyprlandService {}
impl EventEmitter<ActiveWindowChanged> for HyprlandService {}
impl EventEmitter<FullscreenChanged> for HyprlandService {}

// Data structure to send from background worker to UI thread
enum HyprlandUpdate {
    Workspace(Vec<Workspace>, i32),
    Window(Option<ActiveWindow>),
    Fullscreen(Option<i32>), // workspace id with fullscreen, or None
}

impl HyprlandService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let workspaces = Arc::new(RwLock::new(Vec::new()));
        let active_workspace_id = Arc::new(RwLock::new(1));
        let active_window = Arc::new(RwLock::new(None));
        let fullscreen_workspace = Arc::new(RwLock::new(None));

        // Create channel for communication: Worker (Tokio) -> UI (GPUI)
        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<HyprlandUpdate>();

        // 1. Worker Task (Tokio Runtime): Handles I/O and process execution
        gpui_tokio::Tokio::spawn(cx, async move { Self::hyprland_worker(ui_tx).await }).detach();

        // 2. UI Task (GPUI Executor): Receives updates and mutates state
        let workspaces_clone = Arc::clone(&workspaces);
        let active_workspace_id_clone = Arc::clone(&active_workspace_id);
        let active_window_clone = Arc::clone(&active_window);
        let fullscreen_workspace_clone = Arc::clone(&fullscreen_workspace);

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(update) = ui_rx.next().await {
                    let mut ws_changed = false;
                    let mut win_changed = false;
                    let mut fs_changed = None;

                    match update {
                        HyprlandUpdate::Workspace(new_ws, new_id) => {
                            let mut ws = workspaces_clone.write();
                            let mut id = active_workspace_id_clone.write();
                            if *ws != new_ws || *id != new_id {
                                *ws = new_ws;
                                *id = new_id;
                                ws_changed = true;
                            }
                        }
                        HyprlandUpdate::Window(new_win) => {
                            let mut win = active_window_clone.write();
                            if *win != new_win {
                                *win = new_win;
                                win_changed = true;
                            }
                        }
                        HyprlandUpdate::Fullscreen(fs_ws) => {
                            let mut current = fullscreen_workspace_clone.write();
                            if *current != fs_ws {
                                let active_ws = *active_workspace_id_clone.read();
                                let was_fullscreen_here = *current == Some(active_ws);
                                let is_fullscreen_here = fs_ws == Some(active_ws);
                                *current = fs_ws;
                                // Only emit if fullscreen state changed on current workspace
                                if was_fullscreen_here != is_fullscreen_here {
                                    fs_changed = Some(is_fullscreen_here);
                                }
                            }
                        }
                    }

                    if ws_changed || win_changed || fs_changed.is_some() {
                        let _ = this.update(&mut cx, |_, cx| {
                            if ws_changed {
                                cx.emit(WorkspaceChanged);
                            }
                            if win_changed {
                                cx.emit(ActiveWindowChanged);
                            }
                            if let Some(fs) = fs_changed {
                                cx.emit(FullscreenChanged(fs));
                            }
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();

        Self {
            workspaces,
            active_workspace_id,
            active_window,
            fullscreen_workspace,
            executor: cx.background_executor().clone(),
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

    pub fn has_fullscreen(&self) -> bool {
        let active_ws = *self.active_workspace_id.read();
        *self.fullscreen_workspace.read() == Some(active_ws)
    }

    pub fn switch_to_workspace(&self, workspace_id: i32) {
        let ws_id = workspace_id.to_string();
        self.executor
            .spawn(async move {
                let _ = std::process::Command::new("hyprctl")
                    .args(["dispatch", "workspace", &ws_id])
                    .output();
            })
            .detach();
    }

    // Worker running in Tokio context
    async fn hyprland_worker(ui_tx: futures::channel::mpsc::UnboundedSender<HyprlandUpdate>) {
        let hypr_sig = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(sig) => sig,
            Err(_) => return,
        };

        let socket_path = format!(
            "/run/user/{}/hypr/{}/.socket2.sock",
            std::env::var("UID").unwrap_or_else(|_| "1000".to_string()),
            hypr_sig
        );

        let (socket_tx, mut socket_rx) = futures::channel::mpsc::unbounded();

        // Dedicated thread for blocking socket read
        std::thread::spawn(move || {
            if let Ok(stream) = UnixStream::connect(&socket_path) {
                let reader = BufReader::new(stream);
                for line in reader.lines().map_while(Result::ok) {
                    if line.starts_with("workspace>>")
                        || line.starts_with("createworkspace>>")
                        || line.starts_with("destroyworkspace>>")
                    {
                        let _ = socket_tx.unbounded_send(0); // workspace
                    } else if line.starts_with("activewindow>>")
                        || line.starts_with("closewindow>>")
                        || line.starts_with("openwindow>>")
                    {
                        let _ = socket_tx.unbounded_send(1); // window
                    } else if line.starts_with("fullscreen>>") {
                        let _ = socket_tx.unbounded_send(2); // fullscreen
                    }
                }
            }
        });

        // Initial fetch
        let (ws, id) = Self::fetch_hyprland_data().await;
        let _ = ui_tx.unbounded_send(HyprlandUpdate::Workspace(ws, id));
        let win = Self::fetch_active_window().await;
        let _ = ui_tx.unbounded_send(HyprlandUpdate::Window(win));
        let fs = Self::fetch_fullscreen_workspace().await;
        let _ = ui_tx.unbounded_send(HyprlandUpdate::Fullscreen(fs));

        // Event loop
        while let Some(ev_type) = socket_rx.next().await {
            // Debounce/Drain similar events
            let mut do_ws = ev_type == 0;
            let mut do_win = ev_type == 1;
            let mut do_fs = ev_type == 2;

            while let Ok(Some(ev)) = socket_rx.try_next() {
                match ev {
                    0 => do_ws = true,
                    1 => do_win = true,
                    2 => do_fs = true,
                    _ => {}
                }
            }

            if do_ws {
                let (ws, id) = Self::fetch_hyprland_data().await;
                let _ = ui_tx.unbounded_send(HyprlandUpdate::Workspace(ws, id));
            }
            if do_win {
                let win = Self::fetch_active_window().await;
                let _ = ui_tx.unbounded_send(HyprlandUpdate::Window(win));
            }
            if do_fs {
                let fs = Self::fetch_fullscreen_workspace().await;
                let _ = ui_tx.unbounded_send(HyprlandUpdate::Fullscreen(fs));
            }
        }
    }

    async fn fetch_hyprland_data() -> (Vec<Workspace>, i32) {
        let workspaces_json = Self::hyprctl(&["workspaces", "-j"]).await;
        let active_workspace_json = Self::hyprctl(&["activeworkspace", "-j"]).await;

        let mut workspaces: Vec<Workspace> =
            serde_json::from_str(&workspaces_json).unwrap_or_default();
        workspaces.sort_by_key(|w| w.id);

        let active_workspace_id: i32 =
            serde_json::from_str::<serde_json::Value>(&active_workspace_json)
                .ok()
                .and_then(|v| v["id"].as_i64())
                .unwrap_or(1) as i32;

        (workspaces, active_workspace_id)
    }

    async fn fetch_active_window() -> Option<ActiveWindow> {
        let active_window_json = Self::hyprctl(&["activewindow", "-j"]).await;
        if active_window_json.trim().is_empty() || active_window_json == "{}" {
            return None;
        }
        serde_json::from_str::<ActiveWindow>(&active_window_json).ok()
    }

    async fn fetch_fullscreen_workspace() -> Option<i32> {
        let json = Self::hyprctl(&["activewindow", "-j"]).await;
        let v: serde_json::Value = serde_json::from_str(&json).ok()?;
        let is_fullscreen = v["fullscreen"].as_i64().map(|v| v > 0).unwrap_or(false);
        if is_fullscreen {
            v["workspace"]["id"].as_i64().map(|id| id as i32)
        } else {
            None
        }
    }

    async fn hyprctl(args: &[&str]) -> String {
        match tokio::process::Command::new("hyprctl")
            .args(args)
            .output()
            .await
        {
            Ok(output) => String::from_utf8(output.stdout).unwrap_or_default(),
            Err(_) => String::new(),
        }
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
