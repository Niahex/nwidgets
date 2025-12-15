use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionType {
    Ethernet,
    Wifi { ssid: String, strength: u8 },
    Vpn { name: String },
    Disconnected,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkState {
    pub connection_type: ConnectionType,
    pub connected: bool,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::Disconnected,
            connected: false,
        }
    }
}

#[derive(Clone)]
pub struct NetworkStateChanged {
    pub state: NetworkState,
}

pub struct NetworkService {
    state: Arc<RwLock<NetworkState>>,
}

impl EventEmitter<NetworkStateChanged> for NetworkService {}

impl NetworkService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = Arc::new(RwLock::new(Self::fetch_network_state()));

        let state_clone = Arc::clone(&state);

        // Poll network state periodically
        cx.spawn(async move |this, mut cx| {
            Self::monitor_network(this, state_clone, &mut cx).await
        })
        .detach();

        Self { state }
    }

    pub fn state(&self) -> NetworkState {
        self.state.read().clone()
    }

    async fn monitor_network(
        this: WeakEntity<Self>,
        state: Arc<RwLock<NetworkState>>,
        cx: &mut AsyncApp,
    ) {
        loop {
            cx.background_executor()
                .timer(Duration::from_secs(3))
                .await;

            let new_state = Self::fetch_network_state();

            let state_changed = {
                let mut current_state = state.write();
                let changed = *current_state != new_state;
                if changed {
                    *current_state = new_state.clone();
                }
                changed
            };

            if state_changed {
                if let Ok(()) = this.update(cx, |_, cx| {
                    cx.emit(NetworkStateChanged { state: new_state });
                    cx.notify();
                }) {}
            }
        }
    }

    fn fetch_network_state() -> NetworkState {
        // Check if we have internet connectivity
        let has_connection = std::process::Command::new("ping")
            .args(["-c", "1", "-W", "1", "8.8.8.8"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !has_connection {
            return NetworkState {
                connection_type: ConnectionType::Disconnected,
                connected: false,
            };
        }

        // Try to detect connection type
        // Check for WiFi
        if let Ok(output) = std::process::Command::new("nmcli")
            .args(["-t", "-f", "ACTIVE,SSID,SIGNAL", "device", "wifi"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 3 && parts[0] == "yes" {
                        let ssid = parts[1].to_string();
                        let strength = parts[2].parse::<u8>().unwrap_or(0);
                        return NetworkState {
                            connection_type: ConnectionType::Wifi { ssid, strength },
                            connected: true,
                        };
                    }
                }
            }
        }

        // Default to Ethernet if connected but not WiFi
        NetworkState {
            connection_type: ConnectionType::Ethernet,
            connected: true,
        }
    }
}

// Global accessor
struct GlobalNetworkService(Entity<NetworkService>);
impl Global for GlobalNetworkService {}

impl NetworkService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalNetworkService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|cx| Self::new(cx));
        cx.set_global(GlobalNetworkService(service.clone()));
        service
    }
}
