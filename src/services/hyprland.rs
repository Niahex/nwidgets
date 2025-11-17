use std::process::Command;
use std::os::unix::net::UnixStream;
use std::io::{Read, Write, BufReader, BufRead};
use std::sync::mpsc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveWindow {
    pub class: String,
    pub title: String,
    #[serde(default)]
    pub initial_class: String,
    #[serde(default)]
    pub initial_title: String,
}

pub struct HyprlandService;

impl HyprlandService {
    pub fn new() -> Self {
        Self
    }

    /// Start monitoring Hyprland events and send workspace + active window updates through the channel
    pub fn start_monitoring() -> mpsc::Receiver<(Vec<Workspace>, i32, Option<ActiveWindow>)> {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            if let Ok(hypr_sig) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
                let socket_path = format!("/run/user/1000/hypr/{}/.socket2.sock", hypr_sig);

                if let Ok(stream) = UnixStream::connect(&socket_path) {
                    let reader = BufReader::new(stream);

                    for line in reader.lines() {
                        if let Ok(line) = line {
                            // Monitor workspace changes
                            if line.starts_with("workspace>>") ||
                               line.starts_with("createworkspace>>") ||
                               line.starts_with("destroyworkspace>>") ||
                               line.starts_with("activewindow>>") ||
                               line.starts_with("closewindow>>") ||
                               line.starts_with("openwindow>>") {
                                let (workspaces, active_workspace) = Self::get_hyprland_data();
                                let active_window = Self::get_active_window();
                                if tx.send((workspaces, active_workspace, active_window)).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        rx
    }

    pub fn get_socket_path() -> Option<String> {
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .ok()
            .map(|sig| format!("/run/user/1000/hypr/{}/.socket.sock", sig))
    }

    pub fn send_command(command: &str) -> Result<String, Box<dyn std::error::Error>> {
        let socket_path = Self::get_socket_path()
            .ok_or("HYPRLAND_INSTANCE_SIGNATURE not found")?;

        let mut stream = UnixStream::connect(socket_path)?;
        stream.write_all(command.as_bytes())?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        Ok(response)
    }

    /// Get the currently active window information
    pub fn get_active_window() -> Option<ActiveWindow> {
        Self::send_command("j/activewindow")
            .ok()
            .and_then(|response| {
                serde_json::from_str::<ActiveWindow>(&response).ok()
            })
    }

    pub fn get_hyprland_data() -> (Vec<Workspace>, i32) {
        let workspaces = Command::new("hyprctl")
            .args(&["workspaces", "-j"])
            .output()
            .and_then(|output| {
                let json_str = String::from_utf8_lossy(&output.stdout);
                serde_json::from_str::<Vec<Workspace>>(&json_str)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            })
            .unwrap_or_default();

        let active_workspace = Command::new("hyprctl")
            .args(&["activeworkspace", "-j"])
            .output()
            .and_then(|output| {
                let json_str = String::from_utf8_lossy(&output.stdout);
                serde_json::from_str::<Workspace>(&json_str)
                    .map(|ws| ws.id)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            })
            .unwrap_or(1);

        (workspaces, active_workspace)
    }
}
