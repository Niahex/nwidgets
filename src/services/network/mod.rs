mod network_state;
mod vpn_manager;
mod wifi_manager;

pub use network_state::{ConnectionType, NetworkState, VpnConnection};
pub use vpn_manager::VpnManager;
pub use wifi_manager::WifiManager;

use futures_util::StreamExt;
use std::sync::mpsc;
use zbus::{proxy, Connection};

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
pub trait ActiveConnection {
    #[zbus(property, name = "Type")]
    fn connection_type(&self) -> zbus::Result<String>;

    #[zbus(property, name = "State")]
    fn state(&self) -> zbus::Result<u32>;

    #[zbus(property, name = "Devices")]
    fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

// NetworkManager Settings interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
pub trait Settings {
    #[zbus(name = "ListConnections")]
    fn list_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

// NetworkManager Settings Connection interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings.Connection",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait SettingsConnection {
    #[zbus(name = "GetSettings")]
    fn get_settings(
        &self,
    ) -> zbus::Result<
        std::collections::HashMap<
            String,
            std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        >,
    >;
}

pub struct NetworkService;

impl NetworkService {
    /// Subscribe a callback to network state changes
    pub fn subscribe_network<F>(callback: F)
    where
        F: Fn(NetworkState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            crate::utils::runtime::block_on(async {
                let connection = match Connection::system().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        eprintln!("Error connecting to system bus: {e}");
                        return;
                    }
                };

                let mut last_state = Self::get_network_state_with_conn(&connection).await.unwrap_or(NetworkState {
                    connected: false,
                    connection_type: ConnectionType::None,
                    signal_strength: 0,
                    ssid: None,
                    vpn_active: false,
                    vpn_connections: Vec::new(),
                });

                if tx.send(last_state.clone()).is_err() {
                    return;
                }

                let nm_proxy = match NetworkManagerProxy::new(&connection).await {
                    Ok(proxy) => proxy,
                    Err(e) => {
                        eprintln!("Error creating NetworkManager proxy: {e}");
                        return;
                    }
                };

                let mut state_changed_stream = Some(nm_proxy.receive_state_changed().await);
                let mut active_connections_stream = Some(nm_proxy.receive_active_connections_changed().await);
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

                loop {
                    let mut should_update = false;

                    tokio::select! {
                        res = async {
                            if let Some(stream) = state_changed_stream.as_mut() {
                                if StreamExt::next(stream).await.is_some() { return true; }
                            }
                            std::future::pending::<bool>().await
                        } => { if res { should_update = true; } },
                        res = async {
                            if let Some(stream) = active_connections_stream.as_mut() {
                                if StreamExt::next(stream).await.is_some() { return true; }
                            }
                            std::future::pending::<bool>().await
                        } => { if res { should_update = true; } },
                        _ = interval.tick() => { should_update = true; }
                    }

                    if should_update {
                        if let Ok(state) = Self::get_network_state_with_conn(&connection).await {
                            if state != last_state {
                                if tx.send(state.clone()).is_err() { break; }
                                last_state = state;
                            }
                        }
                    }
                }
            });
        });

        crate::utils::subscription::ServiceSubscription::subscribe(rx, callback);
    }

    /// Get current network state
    pub async fn get_network_state() -> zbus::Result<NetworkState> {
        let connection = Connection::system().await?;
        Self::get_network_state_with_conn(&connection).await
    }

    async fn get_network_state_with_conn(connection: &Connection) -> zbus::Result<NetworkState> {
        let nm_proxy = NetworkManagerProxy::new(connection).await?;
        let nm_state = nm_proxy.state().await.unwrap_or(0);
        let connected = nm_state >= 50;

        // VPN connections
        let vpn_connections = VpnManager::list_vpn_connections_async(connection).await.unwrap_or_default();
        let vpn_active = vpn_connections.iter().any(|v| v.active);

        if !connected {
            return Ok(NetworkState {
                connected: false,
                connection_type: ConnectionType::None,
                signal_strength: 0,
                ssid: None,
                vpn_active,
                vpn_connections,
            });
        }

        let primary_conn_path = match nm_proxy.primary_connection().await {
            Ok(path) => path,
            Err(_) => {
                return Ok(NetworkState {
                    connected: false,
                    connection_type: ConnectionType::None,
                    signal_strength: 0,
                    ssid: None,
                    vpn_active,
                    vpn_connections,
                });
            }
        };

        let active_conn_proxy = ActiveConnectionProxy::builder(connection).path(primary_conn_path)?.build().await?;
        let conn_type_str = active_conn_proxy.connection_type().await.unwrap_or_default();

        let connection_type = match conn_type_str.as_str() {
            "802-3-ethernet" => ConnectionType::Ethernet,
            "802-11-wireless" => ConnectionType::Wifi,
            _ => ConnectionType::None,
        };

        let (signal_strength, ssid) = if connection_type == ConnectionType::Wifi {
            let devices = active_conn_proxy.devices().await.unwrap_or_default();
            if !devices.is_empty() {
                WifiManager::get_wifi_info(&devices[0]).await.unwrap_or((0, None))
            } else {
                (0, None)
            }
        } else {
            (100, None)
        };

        Ok(NetworkState {
            connected,
            connection_type,
            signal_strength,
            ssid,
            vpn_active,
            vpn_connections,
        })
    }

    pub fn list_vpn_connections() -> Vec<VpnConnection> {
        crate::utils::runtime::block_on(async {
            let connection = match Connection::system().await {
                Ok(conn) => conn,
                Err(_) => return Vec::new(),
            };
            VpnManager::list_vpn_connections_async(&connection).await.unwrap_or_default()
        })
    }

    pub fn connect_vpn(connection_path: &str) {
        println!("Connecting to VPN: {connection_path}");
    }

    pub fn disconnect_vpn(connection_path: &str) {
        println!("Disconnecting from VPN: {connection_path}");
    }
}
