use std::sync::Arc;
use parking_lot::RwLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::Notify;
use serde::Deserialize;

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub monitor: String,
    pub windows: u32,
}

#[derive(Clone, Debug, Default)]
pub struct ActiveWindow {
    pub address: String,
    pub title: String,
    pub class: String,
}

type RedrawCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub struct HyprlandService {
    state: Arc<RwLock<HyprlandState>>,
    notify: Arc<Notify>,
    redraw_callbacks: Arc<RwLock<Vec<RedrawCallback>>>,
}

#[derive(Default)]
struct HyprlandState {
    workspaces: Vec<Workspace>,
    active_workspace: i32,
    active_window: ActiveWindow,
    is_fullscreen: bool,
}

impl HyprlandService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(HyprlandState::default())),
            notify: Arc::new(Notify::new()),
            redraw_callbacks: Arc::new(RwLock::new(Vec::new())),
        };

        service.start_listener();
        service
    }

    fn start_listener(&self) {
        let state = self.state.clone();
        let notify = self.notify.clone();
        let redraw_callbacks = self.redraw_callbacks.clone();

        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::fetch_initial_state(&state).await {
                log::error!("Failed to fetch initial Hyprland state: {}", e);
            }

            notify.notify_waiters();
            Self::trigger_redraws(&redraw_callbacks);

            if let Err(e) = Self::listen_events(state, notify, redraw_callbacks).await {
                log::error!("Hyprland listener error: {}", e);
            }
        });
    }

    fn trigger_redraws(callbacks: &Arc<RwLock<Vec<RedrawCallback>>>) {
        let cbs = callbacks.read();
        for callback in cbs.iter() {
            callback();
        }
    }

    async fn fetch_initial_state(state: &Arc<RwLock<HyprlandState>>) -> anyhow::Result<()> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let socket_path = format!("{}/hypr/{}/.socket.sock", runtime_dir, his);

        let mut stream = UnixStream::connect(&socket_path).await?;
        stream.write_all(b"j/workspaces").await?;

        let mut response = String::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await?;
            if n == 0 {
                break;
            }
            response.push_str(&String::from_utf8_lossy(&buf[..n]));
        }

        let mut workspaces: Vec<Workspace> = serde_json::from_str(&response)?;
        workspaces.sort_by_key(|w| w.id);

        let mut stream = UnixStream::connect(&socket_path).await?;
        stream.write_all(b"j/activeworkspace").await?;

        let mut response = String::new();
        let mut buf = [0u8; 2048];
        loop {
            let n = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await?;
            if n == 0 {
                break;
            }
            response.push_str(&String::from_utf8_lossy(&buf[..n]));
        }

        #[derive(Deserialize)]
        struct ActiveWorkspaceResponse {
            id: i32,
            name: String,
        }

        if let Ok(active_ws) = serde_json::from_str::<ActiveWorkspaceResponse>(&response) {
            let active_id = active_ws.id;
            state.write().active_workspace = active_id;
            
            if !workspaces.iter().any(|w| w.id == active_id) {
                workspaces.push(Workspace {
                    id: active_id,
                    name: active_ws.name,
                    monitor: String::new(),
                    windows: 0,
                });
                workspaces.sort_by_key(|w| w.id);
            }
        }

        let mut stream = UnixStream::connect(&socket_path).await?;
        stream.write_all(b"/activewindow").await?;

        let mut response = String::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await?;
            if n == 0 {
                break;
            }
            response.push_str(&String::from_utf8_lossy(&buf[..n]));
        }

        for line in response.lines() {
            if let Some(class_str) = line.strip_prefix("class: ") {
                state.write().active_window.class = class_str.to_string();
            } else if let Some(title_str) = line.strip_prefix("title: ") {
                state.write().active_window.title = title_str.to_string();
            }
        }

        state.write().workspaces = workspaces;
        Ok(())
    }

    async fn listen_events(state: Arc<RwLock<HyprlandState>>, notify: Arc<Notify>, redraw_callbacks: Arc<RwLock<Vec<RedrawCallback>>>) -> anyhow::Result<()> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let socket_path = format!("{}/hypr/{}/.socket2.sock", runtime_dir, his);

        let stream = UnixStream::connect(&socket_path).await?;
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            Self::handle_event(&state, &line, &notify, &redraw_callbacks);
        }

        Ok(())
    }

    fn handle_event(state: &Arc<RwLock<HyprlandState>>, event: &str, notify: &Arc<Notify>, redraw_callbacks: &Arc<RwLock<Vec<RedrawCallback>>>) {
        let parts: Vec<&str> = event.splitn(2, ">>").collect();
        if parts.len() != 2 {
            return;
        }

        let event_type = parts[0];
        let data = parts[1];

        match event_type {
            "workspace" => {
                if let Ok(id) = data.parse::<i32>() {
                    state.write().active_workspace = id;
                    notify.notify_waiters();
                    Self::trigger_redraws(redraw_callbacks);
                }
            }
            "activewindow" => {
                let parts: Vec<&str> = data.splitn(2, ",").collect();
                if parts.len() == 2 {
                    let mut s = state.write();
                    s.active_window.class = parts[0].to_string();
                    s.active_window.title = parts[1].to_string();
                }
                notify.notify_waiters();
                Self::trigger_redraws(redraw_callbacks);
            }
            "fullscreen" => {
                state.write().is_fullscreen = data == "1";
                notify.notify_waiters();
                Self::trigger_redraws(redraw_callbacks);
            }
            "createworkspace" => {
                if let Ok(id) = data.parse::<i32>() {
                    let mut s = state.write();
                    if !s.workspaces.iter().any(|w| w.id == id) {
                        s.workspaces.push(Workspace {
                            id,
                            name: id.to_string(),
                            monitor: String::new(),
                            windows: 0,
                        });
                        s.workspaces.sort_by_key(|w| w.id);
                    }
                }
                notify.notify_waiters();
                Self::trigger_redraws(redraw_callbacks);
            }
            "destroyworkspace" => {
                if let Ok(id) = data.parse::<i32>() {
                    state.write().workspaces.retain(|ws| ws.id != id);
                }
                notify.notify_waiters();
                Self::trigger_redraws(redraw_callbacks);
            }
            _ => {}
        }
    }

    pub fn subscribe(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    pub fn on_change<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.redraw_callbacks.write().push(Arc::new(callback));
    }

    pub fn get_active_workspace(&self) -> i32 {
        self.state.read().active_workspace
    }

    pub fn get_workspaces(&self) -> Vec<Workspace> {
        let state = self.state.read();
        let mut workspaces = state.workspaces.clone();
        
        let active_id = state.active_workspace;
        if !workspaces.iter().any(|w| w.id == active_id) {
            workspaces.push(Workspace {
                id: active_id,
                name: active_id.to_string(),
                monitor: String::new(),
                windows: 0,
            });
            workspaces.sort_by_key(|w| w.id);
        }
        
        workspaces
    }

    pub fn get_active_window(&self) -> ActiveWindow {
        self.state.read().active_window.clone()
    }

    pub fn is_fullscreen(&self) -> bool {
        self.state.read().is_fullscreen
    }

    pub fn switch_workspace(&self, workspace: i32) {
        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::do_switch_workspace(workspace).await {
                log::error!("Failed to switch workspace: {}", e);
            }
        });
    }

    async fn do_switch_workspace(workspace: i32) -> anyhow::Result<()> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let socket_path = format!("{}/hypr/{}/.socket.sock", runtime_dir, his);

        let mut stream = UnixStream::connect(&socket_path).await?;
        let command = format!("/dispatch workspace {}", workspace);
        stream.write_all(command.as_bytes()).await?;

        Ok(())
    }
}
