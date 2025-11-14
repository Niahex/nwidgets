use std::process::Command;
use std::os::unix::net::UnixStream;
use std::io::{Read, Write, BufReader, BufRead};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
}

pub struct HyprlandService;

impl HyprlandService {
    pub fn new() -> Self {
        println!("[HYPRLAND_SERVICE] 🖥️  Creating HyprlandService");
        Self
    }

    /// Start monitoring Hyprland events and send workspace updates through the channel
    pub fn start_monitoring() -> mpsc::UnboundedReceiver<(Vec<Workspace>, i32)> {
        println!("[HYPRLAND_SERVICE] 🔍 Starting Hyprland monitoring");
        let (tx, rx) = mpsc::unbounded_channel();

        std::thread::spawn(move || {
            println!("[HYPRLAND_SERVICE] 🧵 Monitor thread started");

            if let Ok(hypr_sig) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
                let socket_path = format!("/run/user/1000/hypr/{}/.socket2.sock", hypr_sig);
                println!("[HYPRLAND_SERVICE] 🔌 Connecting to socket: {}", socket_path);

                match UnixStream::connect(&socket_path) {
                    Ok(stream) => {
                        println!("[HYPRLAND_SERVICE] ✅ Connected to Hyprland socket");
                        let reader = BufReader::new(stream);

                        for line in reader.lines() {
                            if let Ok(line) = line {
                                println!("[HYPRLAND_SERVICE] 📡 Event: {}", line);

                                if line.starts_with("workspace>>") ||
                                   line.starts_with("createworkspace>>") ||
                                   line.starts_with("destroyworkspace>>") {
                                    let data = Self::get_hyprland_data();
                                    println!("[HYPRLAND_SERVICE] 🔔 Sending workspace update");

                                    if tx.send(data).is_err() {
                                        println!("[HYPRLAND_SERVICE] ⚠️  Receiver dropped, stopping monitoring");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("[HYPRLAND_SERVICE] ❌ Failed to connect to socket: {}", e);
                    }
                }
            } else {
                println!("[HYPRLAND_SERVICE] ⚠️  HYPRLAND_INSTANCE_SIGNATURE not set");
            }
        });

        rx
    }

    pub fn get_socket_path() -> Option<String> {
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .ok()
            .map(|sig| format!("/run/user/1000/hypr/{}/hyprland.sock", sig))
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

    pub fn get_hyprland_data() -> (Vec<Workspace>, i32) {
        println!("[HYPRLAND_SERVICE] 🖥️  Fetching workspaces data");
        let workspaces = Command::new("hyprctl")
            .args(&["workspaces", "-j"])
            .output()
            .and_then(|output| {
                let json_str = String::from_utf8_lossy(&output.stdout);
                println!("[HYPRLAND_SERVICE] 📄 Workspaces JSON: {}", json_str.trim());
                serde_json::from_str::<Vec<Workspace>>(&json_str)
                    .map_err(|e| {
                        println!("[HYPRLAND_SERVICE] ❌ Failed to parse workspaces: {}", e);
                        std::io::Error::new(std::io::ErrorKind::Other, e)
                    })
            })
            .unwrap_or_else(|e| {
                println!("[HYPRLAND_SERVICE] ❌ Failed to get workspaces: {}", e);
                Vec::new()
            });

        println!("[HYPRLAND_SERVICE] 📊 Found {} workspaces", workspaces.len());

        let active_workspace = Command::new("hyprctl")
            .args(&["activeworkspace", "-j"])
            .output()
            .and_then(|output| {
                let json_str = String::from_utf8_lossy(&output.stdout);
                println!("[HYPRLAND_SERVICE] 📄 Active workspace JSON: {}", json_str.trim());
                serde_json::from_str::<Workspace>(&json_str)
                    .map_err(|e| {
                        println!("[HYPRLAND_SERVICE] ❌ Failed to parse active workspace: {}", e);
                        std::io::Error::new(std::io::ErrorKind::Other, e)
                    })
            })
            .map(|ws| {
                println!("[HYPRLAND_SERVICE] ✅ Active workspace: id={}, name={}", ws.id, ws.name);
                ws.id
            })
            .unwrap_or_else(|e| {
                println!("[HYPRLAND_SERVICE] ❌ Failed to get active workspace: {} - using default: 1", e);
                1
            });

        println!("[HYPRLAND_SERVICE] 📊 Returning {} workspaces, active={}", workspaces.len(), active_workspace);
        (workspaces, active_workspace)
    }
}
