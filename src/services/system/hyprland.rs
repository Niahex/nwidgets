use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, BackgroundExecutor, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;

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
    open_windows: Arc<RwLock<Vec<String>>>, // Track open window classes
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
    WindowOpened(String),    // window class
    WindowClosed(String),    // window class
}

impl HyprlandService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let workspaces = Arc::new(RwLock::new(Vec::with_capacity(10)));
        let active_workspace_id = Arc::new(RwLock::new(1));
        let active_window = Arc::new(RwLock::new(None));
        let fullscreen_workspace = Arc::new(RwLock::new(None));
        let open_windows = Arc::new(RwLock::new(Vec::with_capacity(20)));

        // Create channel for communication: Worker (Tokio) -> UI (GPUI)
        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<HyprlandUpdate>();

        // 1. Worker Task (Tokio Runtime): Handles I/O and process execution
        gpui_tokio::Tokio::spawn(cx, async move { Self::hyprland_worker(ui_tx).await }).detach();

        // 2. UI Task (GPUI Executor): Receives updates and mutates state
        let workspaces_clone = Arc::clone(&workspaces);
        let active_workspace_id_clone = Arc::clone(&active_workspace_id);
        let active_window_clone = Arc::clone(&active_window);
        let fullscreen_workspace_clone = Arc::clone(&fullscreen_workspace);
        let open_windows_clone = Arc::clone(&open_windows);

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(update) = ui_rx.next().await {
                    let mut ws_changed = false;
                    let mut win_changed = false;
                    let mut fs_changed = None;

                    match update {
                        HyprlandUpdate::Workspace(new_workspaces, new_workspace_id) => {
                            let mut workspaces = workspaces_clone.write();
                            let mut workspace_id = active_workspace_id_clone.write();
                            if *workspaces != new_workspaces || *workspace_id != new_workspace_id {
                                *workspaces = new_workspaces;
                                *workspace_id = new_workspace_id;
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
                                if was_fullscreen_here != is_fullscreen_here {
                                    fs_changed = Some(is_fullscreen_here);
                                }
                            }
                        }
                        HyprlandUpdate::WindowOpened(class) => {
                            let mut windows = open_windows_clone.write();
                            if !windows.contains(&class) {
                                windows.push(class.clone());
                                log::debug!("Window opened: {}", class);
                            }
                        }
                        HyprlandUpdate::WindowClosed(class) => {
                            let mut windows = open_windows_clone.write();
                            windows.retain(|w| w != &class);
                            log::debug!("Window closed: {}", class);
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
            open_windows,
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

    pub fn is_window_open(&self, class: &str) -> bool {
        self.open_windows
            .read()
            .iter()
            .any(|w| w.to_lowercase() == class.to_lowercase())
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

        // Async socket reader using tokio
        tokio::spawn(async move {
            if let Ok(stream) = tokio::net::UnixStream::connect(&socket_path).await {
                let reader = tokio::io::BufReader::new(stream);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if line.starts_with("workspace>>")
                        || line.starts_with("createworkspace>>")
                        || line.starts_with("destroyworkspace>>")
                    {
                        let _ = socket_tx.unbounded_send((0, line)); // workspace
                    } else if line.starts_with("activewindow>>") {
                        let _ = socket_tx.unbounded_send((1, line)); // window
                    } else if line.starts_with("openwindow>>") {
                        let _ = socket_tx.unbounded_send((2, line)); // window opened
                    } else if line.starts_with("closewindow>>") {
                        let _ = socket_tx.unbounded_send((3, line)); // window closed
                    } else if line.starts_with("fullscreen>>") {
                        let _ = socket_tx.unbounded_send((4, line)); // fullscreen
                    }
                }
            }
        });

        // Initial fetch
        let (workspaces, workspace_id) = Self::fetch_hyprland_data().await;
        let _ = ui_tx.unbounded_send(HyprlandUpdate::Workspace(workspaces, workspace_id));
        let window = Self::fetch_active_window().await;
        let _ = ui_tx.unbounded_send(HyprlandUpdate::Window(window));
        let fullscreen = Self::fetch_fullscreen_workspace().await;
        let _ = ui_tx.unbounded_send(HyprlandUpdate::Fullscreen(fullscreen));

        // Event loop
        while let Some((ev_type, line)) = socket_rx.next().await {
            match ev_type {
                0 => {
                    // Workspace event
                    let (workspaces, workspace_id) = Self::fetch_hyprland_data().await;
                    let _ = ui_tx.unbounded_send(HyprlandUpdate::Workspace(workspaces, workspace_id));
                }
                1 => {
                    // Active window event
                    let window = Self::fetch_active_window().await;
                    let _ = ui_tx.unbounded_send(HyprlandUpdate::Window(window));
                }
                2 => {
                    // Window opened: openwindow>>ADDRESS,WORKSPACE,CLASS,TITLE
                    if let Some(data) = line.strip_prefix("openwindow>>") {
                        let parts: Vec<&str> = data.split(',').collect();
                        if parts.len() >= 3 {
                            let class = parts[2].to_string();
                            let _ = ui_tx.unbounded_send(HyprlandUpdate::WindowOpened(class));
                        }
                    }
                }
                3 => {
                    // Window closed: closewindow>>ADDRESS
                    // We need to get the class from hyprctl since we only have address
                    // For now, just trigger active window update
                    let window = Self::fetch_active_window().await;
                    let _ = ui_tx.unbounded_send(HyprlandUpdate::Window(window));
                }
                4 => {
                    // Fullscreen event
                    let fullscreen = Self::fetch_fullscreen_workspace().await;
                    let _ = ui_tx.unbounded_send(HyprlandUpdate::Fullscreen(fullscreen));
                }
                _ => {}
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
