use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug, Default)]
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

#[derive(Clone)]
pub struct HyprlandService {
    state: Arc<RwLock<HyprlandState>>,
}

#[derive(Default)]
struct HyprlandState {
    occupied_workspaces: HashSet<i32>,
    active_workspace: i32,
    active_window: ActiveWindow,
    is_fullscreen: bool,
}

impl HyprlandService {
    pub fn new() -> Self {
        log::info!("Initializing HyprlandService");
        let service = Self {
            state: Arc::new(RwLock::new(HyprlandState::default())),
        };

        service.start_listener();
        service
    }

    fn start_listener(&self) {
        let state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::fetch_initial_state(&state).await {
                log::error!("Failed to fetch initial Hyprland state: {}", e);
            }

            if let Err(e) = Self::listen_events(state).await {
                log::error!("Hyprland listener error: {}", e);
            }
        });
    }

    async fn fetch_initial_state(state: &Arc<RwLock<HyprlandState>>) -> anyhow::Result<()> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        let socket_path = format!("/tmp/hypr/{}/.socket.sock", his);
        log::info!("Fetching initial Hyprland state from {}", socket_path);

        let mut stream = UnixStream::connect(&socket_path).await?;
        stream.write_all(b"/workspaces").await?;

        let mut response = String::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await?;
            if n == 0 {
                break;
            }
            response.push_str(&String::from_utf8_lossy(&buf[..n]));
        }

        let mut occupied = HashSet::new();
        for line in response.lines() {
            if let Some(id_str) = line.strip_prefix("workspace ID ") {
                if let Some(id_part) = id_str.split_whitespace().next() {
                    if let Ok(id) = id_part.parse::<i32>() {
                        occupied.insert(id);
                    }
                }
            }
        }

        let mut stream = UnixStream::connect(&socket_path).await?;
        stream.write_all(b"/activeworkspace").await?;

        let mut response = String::new();
        let mut buf = [0u8; 1024];
        loop {
            let n = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await?;
            if n == 0 {
                break;
            }
            response.push_str(&String::from_utf8_lossy(&buf[..n]));
        }

        if let Some(id_str) = response.strip_prefix("workspace ID ") {
            if let Some(id_part) = id_str.split_whitespace().next() {
                if let Ok(id) = id_part.parse::<i32>() {
                    state.write().active_workspace = id;
                }
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

        state.write().occupied_workspaces = occupied;
        log::info!("Initial state loaded: workspace={}, window={} - {}", 
            state.read().active_workspace,
            state.read().active_window.class,
            state.read().active_window.title);
        Ok(())
    }

    async fn listen_events(state: Arc<RwLock<HyprlandState>>) -> anyhow::Result<()> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let socket_path = format!("{}/hypr/{}/.socket2.sock", runtime_dir, his);

        let stream = UnixStream::connect(&socket_path).await?;
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            Self::handle_event(&state, &line);
        }

        Ok(())
    }

    fn handle_event(state: &Arc<RwLock<HyprlandState>>, event: &str) {
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
                }
            }
            "activewindow" => {
                let parts: Vec<&str> = data.splitn(2, ",").collect();
                if parts.len() == 2 {
                    let mut s = state.write();
                    s.active_window.class = parts[0].to_string();
                    s.active_window.title = parts[1].to_string();
                    log::info!("Active window changed: {} - {}", parts[0], parts[1]);
                }
            }
            "fullscreen" => {
                state.write().is_fullscreen = data == "1";
            }
            "createworkspace" => {
                if let Ok(id) = data.parse::<i32>() {
                    state.write().occupied_workspaces.insert(id);
                }
            }
            "destroyworkspace" => {
                if let Ok(id) = data.parse::<i32>() {
                    state.write().occupied_workspaces.remove(&id);
                }
            }
            _ => {}
        }
    }

    pub fn get_active_workspace(&self) -> i32 {
        self.state.read().active_workspace
    }

    pub fn get_occupied_workspaces(&self) -> HashSet<i32> {
        self.state.read().occupied_workspaces.clone()
    }

    pub fn get_active_window(&self) -> ActiveWindow {
        let window = self.state.read().active_window.clone();
        log::debug!("get_active_window: {} - {}", window.class, window.title);
        window
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
