use std::collections::HashMap;
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
    workspaces: HashMap<i32, Workspace>,
    active_workspace: i32,
    active_window: ActiveWindow,
    is_fullscreen: bool,
}

impl HyprlandService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(HyprlandState::default())),
        };

        service.start_listener();
        service
    }

    fn start_listener(&self) {
        let state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::listen_events(state).await {
                log::error!("Hyprland listener error: {}", e);
            }
        });
    }

    async fn listen_events(state: Arc<RwLock<HyprlandState>>) -> anyhow::Result<()> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        let socket_path = format!("/tmp/hypr/{}/.socket2.sock", his);

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
                }
            }
            "fullscreen" => {
                state.write().is_fullscreen = data == "1";
            }
            "createworkspace" | "destroyworkspace" => {
            }
            _ => {}
        }
    }

    pub fn get_active_workspace(&self) -> i32 {
        self.state.read().active_workspace
    }

    pub fn get_active_window(&self) -> ActiveWindow {
        self.state.read().active_window.clone()
    }

    pub fn is_fullscreen(&self) -> bool {
        self.state.read().is_fullscreen
    }

    pub async fn switch_workspace(workspace: i32) -> anyhow::Result<()> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        let socket_path = format!("/tmp/hypr/{}/.socket.sock", his);

        let mut stream = UnixStream::connect(&socket_path).await?;
        let command = format!("/dispatch workspace {}", workspace);
        stream.write_all(command.as_bytes()).await?;

        Ok(())
    }
}
