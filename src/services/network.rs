use zbus::{Connection, proxy};
use std::sync::mpsc;
use glib::MainContext;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    Wifi,
    Ethernet,
    None,
}

#[derive(Debug, Clone)]
pub struct NetworkState {
    pub connected: bool,
    pub connection_type: ConnectionType,
    pub signal_strength: u8, // 0-100, only relevant for WiFi
    pub ssid: Option<String>, // WiFi SSID if connected
    pub vpn_active: bool, // true if VPN connection is active
}

#[derive(Debug, Clone)]
pub struct VpnConnection {
    pub id: String,
    pub name: String,
    pub vpn_type: String, // "openvpn", "wireguard", etc.
    pub active: bool,
    pub path: String, // D-Bus path for the connection
}

impl NetworkState {
    pub fn get_icon_name(&self) -> &'static str {
        if !self.connected {
            match self.connection_type {
                ConnectionType::Ethernet => "network-eternet-disconnected",
                _ => "network-eternet-disconnected",
            }
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

                    // Format: network-wifi-{signal}-{secure/unsecure}
                    // Ex: network-wifi-high-secure
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

// NetworkManager Wireless Device interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
trait WirelessDevice {
    #[zbus(property, name = "ActiveAccessPoint")]
    fn active_access_point(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

// NetworkManager AccessPoint interface
#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
trait AccessPoint {
    #[zbus(property, name = "Strength")]
    fn strength(&self) -> zbus::Result<u8>;

    #[zbus(property, name = "Ssid")]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;
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
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
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

        // Create async channel to execute callback on main thread
        let (async_tx, async_rx) = async_channel::unbounded();

        // Thread that receives updates and transfers them to the async channel
        std::thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                if async_tx.send_blocking(state).is_err() {
                    break;
                }
            }
        });

        // Attach callback to async channel
        MainContext::default().spawn_local(async move {
            while let Ok(state) = async_rx.recv().await {
                callback(state);
            }
        });
    }

    /// Get current network state
    pub async fn get_network_state() -> zbus::Result<NetworkState> {
        let connection = Connection::system().await?;
        let nm_proxy = NetworkManagerProxy::new(&connection).await?;

        // Get NetworkManager state
        let nm_state = nm_proxy.state().await.unwrap_or(0);

        // NM_STATE values:
        // 10 = DISCONNECTED
        // 20 = ASLEEP
        // 30 = DISCONNECTING
        // 40 = CONNECTING
        // 50 = CONNECTED_LOCAL
        // 60 = CONNECTED_SITE
        // 70 = CONNECTED_GLOBAL
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

        // Check if VPN is active by looking at all active connections
        let vpn_active = Self::check_vpn_active(&connection, &nm_proxy).await;

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
            Self::get_wifi_info(&connection, &active_conn_proxy).await
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

    async fn check_vpn_active(connection: &Connection, nm_proxy: &NetworkManagerProxy<'_>) -> bool {
        // Get all active connections
        let active_connections = match nm_proxy.active_connections().await {
            Ok(conns) => conns,
            Err(_) => return false,
        };

        // Check if any connection is a VPN type
        for conn_path in active_connections {
            if let Some(builder) = ActiveConnectionProxy::builder(connection)
                .path(conn_path)
                .ok()
            {
                if let Ok(proxy) = builder.build().await {
                    if let Ok(conn_type) = proxy.connection_type().await {
                        // VPN connection types in NetworkManager
                        if conn_type.contains("vpn")
                            || conn_type == "wireguard"
                            || conn_type == "openvpn"
                            || conn_type == "vpnc"
                            || conn_type == "pptp"
                            || conn_type == "l2tp" {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    async fn get_wifi_info(
        connection: &Connection,
        active_conn_proxy: &ActiveConnectionProxy<'_>,
    ) -> (u8, Option<String>) {
        // Get device from active connection
        let devices = match active_conn_proxy.devices().await {
            Ok(devs) => devs,
            Err(_) => return (0, None),
        };

        if devices.is_empty() {
            return (0, None);
        }

        let device_path = &devices[0];

        // Get wireless device
        let wireless_proxy = match WirelessDeviceProxy::builder(connection)
            .path(device_path.clone())
            .ok()
        {
            Some(builder) => match builder.build().await {
                Ok(proxy) => proxy,
                Err(_) => return (0, None),
            },
            None => return (0, None),
        };

        // Get active access point
        let ap_path = match wireless_proxy.active_access_point().await {
            Ok(path) => path,
            Err(_) => return (0, None),
        };

        // Get access point info
        let ap_proxy = match AccessPointProxy::builder(connection)
            .path(ap_path)
            .ok()
        {
            Some(builder) => match builder.build().await {
                Ok(proxy) => proxy,
                Err(_) => return (0, None),
            },
            None => return (0, None),
        };

        let strength = ap_proxy.strength().await.unwrap_or(0);
        let ssid_bytes = ap_proxy.ssid().await.unwrap_or_default();
        let ssid = if !ssid_bytes.is_empty() {
            String::from_utf8(ssid_bytes).ok()
        } else {
            None
        };

        (strength, ssid)
    }

    /// List all VPN connections
    pub fn list_vpn_connections() -> Vec<VpnConnection> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            Self::list_vpn_connections_async().await.unwrap_or_default()
        })
    }

    async fn list_vpn_connections_async() -> zbus::Result<Vec<VpnConnection>> {
        let connection = Connection::system().await?;
        let settings_proxy = SettingsProxy::new(&connection).await?;
        let nm_proxy = NetworkManagerProxy::new(&connection).await?;

        // Get all active connection paths
        let active_connections = nm_proxy.active_connections().await.unwrap_or_default();
        let mut active_vpn_paths = std::collections::HashSet::new();

        for active_conn_path in &active_connections {
            if let Ok(active_conn_proxy) = ActiveConnectionProxy::builder(&connection)
                .path(active_conn_path.clone())?
                .build()
                .await
            {
                if let Ok(conn_type) = active_conn_proxy.connection_type().await {
                    if conn_type.contains("vpn") || conn_type == "wireguard" {
                        active_vpn_paths.insert(active_conn_path.to_string());
                    }
                }
            }
        }

        // Get all configured connections
        let connections = settings_proxy.list_connections().await?;
        let mut vpn_connections = Vec::new();

        for conn_path in connections {
            if let Ok(conn_proxy) = SettingsConnectionProxy::builder(&connection)
                .path(conn_path.clone())?
                .build()
                .await
            {
                if let Ok(settings) = conn_proxy.get_settings().await {
                    // Check if this is a VPN connection
                    if let Some(connection_settings) = settings.get("connection") {
                        if let Some(type_value) = connection_settings.get("type") {
                            if let Ok(conn_type) = type_value.downcast_ref::<zbus::zvariant::Str>() {
                                let conn_type_str = conn_type.as_str();

                                // Check if it's a VPN type
                                if conn_type_str.contains("vpn") || conn_type_str == "wireguard" {
                                    let name = if let Some(id_value) = connection_settings.get("id") {
                                        if let Ok(id) = id_value.downcast_ref::<zbus::zvariant::Str>() {
                                            id.to_string()
                                        } else {
                                            "Unknown VPN".to_string()
                                        }
                                    } else {
                                        "Unknown VPN".to_string()
                                    };

                                    let uuid = if let Some(uuid_value) = connection_settings.get("uuid") {
                                        if let Ok(uuid) = uuid_value.downcast_ref::<zbus::zvariant::Str>() {
                                            uuid.to_string()
                                        } else {
                                            String::new()
                                        }
                                    } else {
                                        String::new()
                                    };

                                    // Check if this connection is currently active
                                    let active = active_vpn_paths.contains(&conn_path.to_string());

                                    vpn_connections.push(VpnConnection {
                                        id: uuid,
                                        name,
                                        vpn_type: conn_type_str.to_string(),
                                        active,
                                        path: conn_path.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(vpn_connections)
    }

    /// Activate a VPN connection
    pub fn connect_vpn(connection_path: &str) {
        let path = connection_path.to_string();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = Self::connect_vpn_async(&path).await {
                    eprintln!("Failed to connect VPN: {}", e);
                }
            });
        });
    }

    async fn connect_vpn_async(connection_path: &str) -> zbus::Result<()> {
        let connection = Connection::system().await?;
        let nm_proxy = NetworkManagerProxy::new(&connection).await?;

        // ActivateConnection method
        connection.call_method(
            Some("org.freedesktop.NetworkManager"),
            "/org/freedesktop/NetworkManager",
            Some("org.freedesktop.NetworkManager"),
            "ActivateConnection",
            &(
                zbus::zvariant::ObjectPath::try_from(connection_path)?,
                zbus::zvariant::ObjectPath::try_from("/")?,
                zbus::zvariant::ObjectPath::try_from("/")?,
            ),
        ).await?;

        Ok(())
    }

    /// Deactivate a VPN connection
    pub fn disconnect_vpn(connection_path: &str) {
        let path = connection_path.to_string();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = Self::disconnect_vpn_async(&path).await {
                    eprintln!("Failed to disconnect VPN: {}", e);
                }
            });
        });
    }

    async fn disconnect_vpn_async(connection_path: &str) -> zbus::Result<()> {
        let connection = Connection::system().await?;
        let nm_proxy = NetworkManagerProxy::new(&connection).await?;

        // First, find the active connection for this connection path
        let active_connections = nm_proxy.active_connections().await?;

        for active_conn_path in active_connections {
            // DeactivateConnection method
            connection.call_method(
                Some("org.freedesktop.NetworkManager"),
                "/org/freedesktop/NetworkManager",
                Some("org.freedesktop.NetworkManager"),
                "DeactivateConnection",
                &(active_conn_path,),
            ).await?;
        }

        Ok(())
    }
}
