use std::process::Command;
use std::os::unix::net::UnixStream;
use std::io::{Read, Write, BufReader, BufRead};
use std::sync::{Arc, Mutex, mpsc};
use serde::{Deserialize, Serialize};
use glib::MainContext;
use once_cell::sync::Lazy;

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
    #[serde(default)]
    pub address: String,
}

// Types pour les callbacks
type WorkspaceSender = mpsc::Sender<(Vec<Workspace>, i32)>;
type ActiveWindowSender = mpsc::Sender<Option<ActiveWindow>>;

/// Structure pour gérer le monitoring centralisé de Hyprland
struct HyprlandMonitor {
    workspace_subscribers: Arc<Mutex<Vec<WorkspaceSender>>>,
    active_window_subscribers: Arc<Mutex<Vec<ActiveWindowSender>>>,
    started: Arc<Mutex<bool>>,
}

impl HyprlandMonitor {
    fn new() -> Self {
        Self {
            workspace_subscribers: Arc::new(Mutex::new(Vec::new())),
            active_window_subscribers: Arc::new(Mutex::new(Vec::new())),
            started: Arc::new(Mutex::new(false)),
        }
    }

    fn ensure_started(&self) {
        let mut started = self.started.lock().unwrap();
        if *started {
            return;
        }
        *started = true;

        let workspace_subscribers = Arc::clone(&self.workspace_subscribers);
        let active_window_subscribers = Arc::clone(&self.active_window_subscribers);

        std::thread::spawn(move || {
            if let Ok(hypr_sig) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
                let socket_path = format!("/run/user/1000/hypr/{}/.socket2.sock", hypr_sig);

                if let Ok(stream) = UnixStream::connect(&socket_path) {
                    let reader = BufReader::new(stream);

                    // Envoyer l'état initial
                    Self::broadcast_updates(&workspace_subscribers, &active_window_subscribers);

                    for line in reader.lines() {
                        if let Ok(line) = line {
                            // Monitor workspace et active window changes
                            if line.starts_with("workspace>>") ||
                               line.starts_with("createworkspace>>") ||
                               line.starts_with("destroyworkspace>>") ||
                               line.starts_with("activewindow>>") ||
                               line.starts_with("closewindow>>") ||
                               line.starts_with("openwindow>>") {
                                Self::broadcast_updates(&workspace_subscribers, &active_window_subscribers);
                            }
                        }
                    }
                }
            }
        });
    }

    fn broadcast_updates(
        workspace_subscribers: &Arc<Mutex<Vec<WorkspaceSender>>>,
        active_window_subscribers: &Arc<Mutex<Vec<ActiveWindowSender>>>,
    ) {
        let (workspaces, active_workspace) = HyprlandService::get_hyprland_data();
        let active_window = HyprlandService::get_active_window();

        // Broadcast aux workspace subscribers
        if let Ok(mut subs) = workspace_subscribers.lock() {
            subs.retain(|tx| tx.send((workspaces.clone(), active_workspace)).is_ok());
        }

        // Broadcast aux active window subscribers
        if let Ok(mut subs) = active_window_subscribers.lock() {
            subs.retain(|tx| tx.send(active_window.clone()).is_ok());
        }
    }

    fn add_workspace_subscriber(&self, tx: WorkspaceSender) {
        if let Ok(mut subs) = self.workspace_subscribers.lock() {
            subs.push(tx);
        }
    }

    fn add_active_window_subscriber(&self, tx: ActiveWindowSender) {
        if let Ok(mut subs) = self.active_window_subscribers.lock() {
            subs.push(tx);
        }
    }
}

/// Instance statique globale du moniteur
static MONITOR: Lazy<HyprlandMonitor> = Lazy::new(|| HyprlandMonitor::new());

pub struct HyprlandService;

impl HyprlandService {
    pub fn new() -> Self {
        Self
    }

    /// Abonne un callback aux changements de workspace
    /// Le callback sera appelé sur le thread principal GTK
    pub fn subscribe_workspace<F>(callback: F)
    where
        F: Fn(Vec<Workspace>, i32) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        // Ajouter le sender à la liste des subscribers
        MONITOR.add_workspace_subscriber(tx);

        // Créer un async channel pour exécuter le callback sur le thread principal
        let (async_tx, async_rx) = async_channel::unbounded();

        // Thread qui reçoit les mises à jour et les transfère au async channel
        std::thread::spawn(move || {
            while let Ok((workspaces, active_workspace)) = rx.recv() {
                if async_tx.send_blocking((workspaces, active_workspace)).is_err() {
                    break;
                }
            }
        });

        // Attacher le callback au async channel
        MainContext::default().spawn_local(async move {
            while let Ok((workspaces, active_workspace)) = async_rx.recv().await {
                callback(workspaces, active_workspace);
            }
        });

        // Démarrer le monitoring si ce n'est pas déjà fait
        MONITOR.ensure_started();
    }

    /// Abonne un callback aux changements de la fenêtre active
    /// Le callback sera appelé sur le thread principal GTK
    pub fn subscribe_active_window<F>(callback: F)
    where
        F: Fn(Option<ActiveWindow>) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        // Ajouter le sender à la liste des subscribers
        MONITOR.add_active_window_subscriber(tx);

        // Créer un async channel pour exécuter le callback sur le thread principal
        let (async_tx, async_rx) = async_channel::unbounded();

        // Thread qui reçoit les mises à jour et les transfère au async channel
        std::thread::spawn(move || {
            while let Ok(active_window) = rx.recv() {
                if async_tx.send_blocking(active_window).is_err() {
                    break;
                }
            }
        });

        // Attacher le callback au async channel
        MainContext::default().spawn_local(async move {
            while let Ok(active_window) = async_rx.recv().await {
                callback(active_window);
            }
        });

        // Démarrer le monitoring si ce n'est pas déjà fait
        MONITOR.ensure_started();
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
