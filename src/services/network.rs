use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionType {
    Wifi,
    Ethernet,
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkState {
    pub connected: bool,
    pub connection_type: ConnectionType,
    pub signal_strength: u8,
    pub ssid: Option<String>,
    pub vpn_active: bool,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            connected: false,
            connection_type: ConnectionType::None,
            signal_strength: 0,
            ssid: None,
            vpn_active: false,
        }
    }
}

impl NetworkState {
    pub fn get_icon_name(&self) -> &'static str {
        if !self.connected {
            "network-eternet-disconnected"
        } else {
            match self.connection_type {
                ConnectionType::Ethernet => {
                    if self.vpn_active {
                        "network-eternet-secure"
                    } else {
                        "network-eternet-unsecure"
                    }
                }
                ConnectionType::Wifi => {
                    let signal_level = if self.signal_strength > 75 {
                        "high"
                    } else if self.signal_strength > 50 {
                        "good"
                    } else if self.signal_strength > 25 {
                        "medium"
                    } else {
                        "low"
                    };

                    match (signal_level, self.vpn_active) {
                        ("high", true) => "network-wifi-high-secure",
                        ("high", false) => "network-wifi-high-unsecure",
                        ("good", true) => "network-wifi-good-secure",
                        ("good", false) => "network-wifi-good-unsecure",
                        ("medium", true) => "network-wifi-medium-secure",
                        ("medium", false) => "network-wifi-medium-unsecure",
                        ("low", true) => "network-wifi-low-secure",
                        ("low", false) => "network-wifi-low-unsecure",
                        _ => "network-wifi-low-unsecure",
                    }
                }
                ConnectionType::None => "network-eternet-disconnected",
            }
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
                let _ = this.update(cx, |_, cx| {
                    cx.emit(NetworkStateChanged { state: new_state });
                    cx.notify();
                });
            }
        }
    }

    fn fetch_network_state() -> NetworkState {
        let has_connection = std::process::Command::new("ping")
            .args(["-c", "1", "-W", "1", "8.8.8.8"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !has_connection {
            return NetworkState::default();
        }

        let vpn_active = Self::check_vpn_active();

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
                            connected: true,
                            connection_type: ConnectionType::Wifi,
                            signal_strength: strength,
                            ssid: Some(ssid),
                            vpn_active,
                        };
                    }
                }
            }
        }

        NetworkState {
            connected: true,
            connection_type: ConnectionType::Ethernet,
            signal_strength: 100,
            ssid: None,
            vpn_active,
        }
    }

    fn check_vpn_active() -> bool {
        std::process::Command::new("nmcli")
            .args(["-t", "-f", "TYPE", "connection", "show", "--active"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|output| output.contains("vpn") || output.contains("wireguard"))
            .unwrap_or(false)
    }
}

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
