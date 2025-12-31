use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

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

// --- DBus Proxies ---

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
    
    // Connectivity state: 1=None, 2=Portal, 3=Limited, 4=Full
    #[zbus(property)]
    fn connectivity(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
trait ActiveConnection {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn type_(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn specific_object(&self) -> zbus::Result<OwnedObjectPath>;
    #[zbus(property)]
    fn vpn(&self) -> zbus::Result<bool>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
trait AccessPoint {
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;
}

// --- Service Implementation ---

impl NetworkService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = Arc::new(RwLock::new(NetworkState::default()));
        let state_clone = Arc::clone(&state);

        cx.spawn(async move |this, cx| Self::monitor_network(this, state_clone, cx).await)
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
        // Initialize DBus connection
        let conn = match Connection::system().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[NetworkService] Failed to connect to system bus: {}", e);
                return;
            }
        };

        // Create proxies
        let nm_proxy = match NetworkManagerProxy::new(&conn).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[NetworkService] Failed to create NM proxy: {}", e);
                return;
            }
        };

        loop {
            // Fetch state via DBus
            let new_state = Self::fetch_network_state_dbus(&conn, &nm_proxy).await;

            // Update state if changed
            let state_changed = {
                let mut current_state = state.write();
                if *current_state != new_state {
                    *current_state = new_state.clone();
                    true
                } else {
                    false
                }
            };

            if state_changed {
                let _ = this.update(cx, |_, cx| {
                    cx.emit(NetworkStateChanged { state: new_state });
                    cx.notify();
                });
            }

            // Sleep for 5 seconds before next check
            // We could also listen to signals for faster updates, but 5s polling on DBus is cheap
            cx.background_executor().timer(Duration::from_secs(5)).await;
        }
    }

    async fn fetch_network_state_dbus(conn: &Connection, nm: &NetworkManagerProxy<'_>) -> NetworkState {
        let mut state = NetworkState::default();

        // 1. Check Connectivity
        // 4 = Full (Internet access), similar to ping success
        // We consider "connected" if connectivity >= 4. 
        // If just local (3), we might show it but maybe with a warning? 
        // For now, let's match original ping behavior: ping success = connected.
        // NM Connectivity 4 is "Full".
        if let Ok(connectivity) = nm.connectivity().await {
            state.connected = connectivity >= 4;
        }

        // 2. Iterate Active Connections
        if let Ok(active_paths) = nm.active_connections().await {
            for path in active_paths {
                if let Ok(ac) = ActiveConnectionProxy::new(conn, path).await {
                    
                    // Check if VPN
                    if let Ok(is_vpn) = ac.vpn().await {
                        if is_vpn {
                            state.vpn_active = true;
                        }
                    } else if let Ok(type_str) = ac.type_().await {
                        // Fallback check by type string
                        if type_str == "vpn" || type_str == "wireguard" {
                            state.vpn_active = true;
                        }
                    }

                    // Check Connection Type & Details
                    if let Ok(type_str) = ac.type_().await {
                        if type_str == "802-11-wireless" {
                            state.connection_type = ConnectionType::Wifi;
                            
                            // Get SSID
                            if let Ok(id) = ac.id().await {
                                state.ssid = Some(id);
                            }

                            // Get Signal Strength from AccessPoint
                            if let Ok(ap_path) = ac.specific_object().await {
                                // Specific object might be "/" if not associated yet
                                if ap_path.as_str() != "/" {
                                    if let Ok(ap) = AccessPointProxy::new(conn, ap_path).await {
                                        if let Ok(strength) = ap.strength().await {
                                            state.signal_strength = strength;
                                        }
                                    }
                                }
                            }
                        } else if type_str == "802-3-ethernet" && state.connection_type == ConnectionType::None {
                            // Only set ethernet if we haven't found wifi (prefer wifi details if both active? unlikely)
                            // Actually, if ethernet is active, it's usually primary.
                            state.connection_type = ConnectionType::Ethernet;
                            state.signal_strength = 100;
                        }
                    }
                }
            }
        }
        
        // If we found a connection type but connectivity is unknown/low, we might want to flag it?
        // But logic above sets `connected` based on global connectivity.
        // If we have an active connection but no internet (connectivity < 4), state.connected will be false.
        // This matches "ping failed" behavior.

        state
    }
}

struct GlobalNetworkService(Entity<NetworkService>);
impl Global for GlobalNetworkService {}

impl NetworkService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalNetworkService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalNetworkService(service.clone()));
        service
    }
}