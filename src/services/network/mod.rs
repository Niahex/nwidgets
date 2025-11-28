mod network_state;
mod wifi_manager;
mod ethernet_manager;
mod vpn_manager;

pub use network_state::{NetworkState, ConnectionType, VpnConnection};
pub use wifi_manager::WifiManager;
pub use ethernet_manager::EthernetManager;
pub use vpn_manager::VpnManager;

use zbus::{Connection, proxy};
use std::sync::mpsc;
use glib::MainContext;

// NetworkManager main interface
#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[zbus(property, name = "ActiveConnections")]
    fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    #[zbus(property, name = "PrimaryConnection")]
    fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    #[zbus(property, name = "State")]
    fn state(&self) -> zbus::Result<u32>;
}

// NetworkManager ActiveConnection interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
trait ActiveConnection {
    #[zbus(property, name = "Type")]
    fn connection_type(&self) -> zbus::Result<String>;

    #[zbus(property, name = "State")]
    fn state(&self) -> zbus::Result<u32>;

    #[zbus(property, name = "Devices")]
    fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

// NetworkManager Device interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
trait Device {
    #[zbus(property, name = "DeviceType")]
    fn device_type(&self) -> zbus::Result<u32>;
}

// NetworkManager Settings interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
trait Settings {
    #[zbus(name = "ListConnections")]
    fn list_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

// NetworkManager Settings Connection interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings.Connection",
    default_service = "org.freedesktop.NetworkManager"
)]
trait SettingsConnection {
    #[zbus(name = "GetSettings")]
    fn get_settings(&self) -> zbus::Result<std::collections::HashMap<String, std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;
}

pub struct NetworkService;

impl NetworkService {
    pub fn new() -> Self {
        Self
    }

    /// Subscribe a callback to network state changes
    /// The callback will be called on the GTK main thread
    pub fn subscribe_network<F>(callback: F)
    where
        F: Fn(NetworkState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        // Thread that monitors the network
        std::thread::spawn(move || {
            super::runtime::block_on(async {
                let mut last_state = Self::get_network_state().await.unwrap_or_else(|_| NetworkState {
                    connected: false,
                    connection_type: ConnectionType::None,
                    signal_strength: 0,
                    ssid: None,
                    vpn_active: false,
                });

                // Send initial state
                let _ = tx.send(last_state.clone());

                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                    if let Ok(state) = Self::get_network_state().await {
                        // Only send updates if state changed
                        if state.connected != last_state.connected
                            || state.connection_type != last_state.connection_type
                            || (state.signal_strength as i16 - last_state.signal_strength as i16).abs() > 5
                            || state.ssid != last_state.ssid
                            || state.vpn_active != last_state.vpn_active
                        {
                            if tx.send(state.clone()).is_err() {
                                break;
                            }
                            last_state = state;
                        }
                    }
                }
            });
        });

        // Utiliser l'abstraction de subscription
        super::subscription::ServiceSubscription::subscribe(rx, callback);
    }

    /// Get current network state
    pub async fn get_network_state() -> zbus::Result<NetworkState> {
        let connection = Connection::system().await?;
        let nm_proxy = NetworkManagerProxy::new(&connection).await?;

        // Get NetworkManager state
        let nm_state = nm_proxy.state().await.unwrap_or(0);

        // NM_STATE values:
        // 10 = DISCONNECTED, 20 = ASLEEP, 30 = DISCONNECTING
        // 40 = CONNECTING, 50 = CONNECTED_LOCAL, 60 = CONNECTED_SITE, 70 = CONNECTED_GLOBAL
        let connected = nm_state >= 50;

        if !connected {
            return Ok(NetworkState {
                connected: false,
                connection_type: ConnectionType::None,
                signal_strength: 0,
                ssid: None,
                vpn_active: false,
            });
        }

        // Check if VPN is active
        let vpn_active = VpnManager::check_vpn_active(&connection, &nm_proxy).await;

        // Get primary connection
        let primary_conn_path = match nm_proxy.primary_connection().await {
            Ok(path) => path,
            Err(_) => {
                return Ok(NetworkState {
                    connected: false,
                    connection_type: ConnectionType::None,
                    signal_strength: 0,
                    ssid: None,
                    vpn_active,
                });
            }
        };

        // Get connection type
        let active_conn_proxy = match ActiveConnectionProxy::builder(&connection)
            .path(primary_conn_path)
            .ok()
        {
            Some(builder) => match builder.build().await {
                Ok(proxy) => proxy,
                Err(_) => {
                    return Ok(NetworkState {
                        connected,
                        connection_type: ConnectionType::None,
                        signal_strength: 0,
                        ssid: None,
                        vpn_active,
                    });
                }
            },
            None => {
                return Ok(NetworkState {
                    connected,
                    connection_type: ConnectionType::None,
                    signal_strength: 0,
                    ssid: None,
                    vpn_active,
                });
            }
        };

        let conn_type_str = active_conn_proxy.connection_type().await.unwrap_or_default();

        let connection_type = match conn_type_str.as_str() {
            "802-3-ethernet" => ConnectionType::Ethernet,
            "802-11-wireless" => ConnectionType::Wifi,
            _ => ConnectionType::None,
        };

        // If WiFi, get signal strength and SSID
        let (signal_strength, ssid) = if connection_type == ConnectionType::Wifi {
            let devices = active_conn_proxy.devices().await.unwrap_or_default();
            if !devices.is_empty() {
                WifiManager::get_wifi_info(&devices[0]).await.unwrap_or((0, None))
            } else {
                (0, None)
            }
        } else {
            (100, None) // Ethernet always "full signal"
        };

        Ok(NetworkState {
            connected,
            connection_type,
            signal_strength,
            ssid,
            vpn_active,
        })
    }

    /// List all VPN connections
    pub fn list_vpn_connections() -> Vec<VpnConnection> {
        VpnManager::list_vpn_connections()
    }

    /// Connect to a VPN
    pub fn connect_vpn(connection_path: &str) {
        // Implementation for connecting to VPN
        // This would typically use NetworkManager's ActivateConnection method
        println!("Connecting to VPN: {}", connection_path);
    }

    /// Disconnect from a VPN
    pub fn disconnect_vpn(connection_path: &str) {
        // Implementation for disconnecting from VPN
        // This would typically use NetworkManager's DeactivateConnection method
        println!("Disconnecting from VPN: {}", connection_path);
    }
}
