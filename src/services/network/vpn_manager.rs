use super::network_state::VpnConnection;
use super::{ActiveConnectionProxy, NetworkManagerProxy, SettingsConnectionProxy, SettingsProxy};
use zbus::Connection;

pub struct VpnManager;

impl VpnManager {
    pub async fn check_vpn_active(
        connection: &Connection,
        nm_proxy: &NetworkManagerProxy<'_>,
    ) -> bool {
        // Get all active connections
        let active_connections = match nm_proxy.active_connections().await {
            Ok(conns) => conns,
            Err(_) => return false,
        };

        // Check if any connection is a VPN type
        for conn_path in active_connections {
            if let Ok(builder) = ActiveConnectionProxy::builder(connection)
                .path(conn_path)
            {
                if let Ok(proxy) = builder.build().await {
                    if let Ok(conn_type) = proxy.connection_type().await {
                        // VPN connection types in NetworkManager
                        if conn_type.contains("vpn")
                            || conn_type == "wireguard"
                            || conn_type == "openvpn"
                            || conn_type == "vpnc"
                            || conn_type == "pptp"
                            || conn_type == "l2tp"
                        {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// List all VPN connections
    pub fn list_vpn_connections() -> Vec<VpnConnection> {
        crate::utils::runtime::block_on(async {
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
                            if let Ok(conn_type) = type_value.downcast_ref::<zbus::zvariant::Str>()
                            {
                                let conn_type_str = conn_type.as_str();

                                // Check if it's a VPN type
                                if conn_type_str.contains("vpn") || conn_type_str == "wireguard" {
                                    let name = if let Some(id_value) = connection_settings.get("id")
                                    {
                                        if let Ok(id) =
                                            id_value.downcast_ref::<zbus::zvariant::Str>()
                                        {
                                            id.to_string()
                                        } else {
                                            "Unknown VPN".to_string()
                                        }
                                    } else {
                                        "Unknown VPN".to_string()
                                    };

                                    let uuid =
                                        if let Some(uuid_value) = connection_settings.get("uuid") {
                                            if let Ok(uuid) =
                                                uuid_value.downcast_ref::<zbus::zvariant::Str>()
                                            {
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
}
