use super::network_state::VpnConnection;
use super::{ActiveConnectionProxy, NetworkManagerProxy, SettingsConnectionProxy, SettingsProxy};
use zbus::Connection;

pub struct VpnManager;

impl VpnManager {
    pub async fn check_vpn_active(
        connection: &Connection,
        nm_proxy: &NetworkManagerProxy<'_>,
    ) -> bool {
        let active_connections = match nm_proxy.active_connections().await {
            Ok(conns) => conns,
            Err(_) => return false,
        };

        for conn_path in active_connections {
            if let Ok(builder) = ActiveConnectionProxy::builder(connection).path(conn_path) {
                if let Ok(proxy) = builder.build().await {
                    if let Ok(conn_type) = proxy.connection_type().await {
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

    pub async fn list_vpn_connections_async(connection: &Connection) -> zbus::Result<Vec<VpnConnection>> {
        let settings_proxy = SettingsProxy::new(connection).await?;
        let nm_proxy = NetworkManagerProxy::new(connection).await?;

        let active_connections = nm_proxy.active_connections().await.unwrap_or_default();
        let mut active_vpn_paths = std::collections::HashSet::new();

        for active_conn_path in &active_connections {
            if let Ok(active_conn_proxy) = ActiveConnectionProxy::builder(connection)
                .path(active_conn_path.clone())
            {
                if let Ok(proxy) = active_conn_proxy.build().await {
                    if let Ok(conn_type) = proxy.connection_type().await {
                        if conn_type.contains("vpn") || conn_type == "wireguard" {
                            active_vpn_paths.insert(active_conn_path.to_string());
                        }
                    }
                }
            }
        }

        let connections = settings_proxy.list_connections().await?;
        let mut vpn_connections = Vec::new();

        for conn_path in connections {
            if let Ok(conn_proxy) = SettingsConnectionProxy::builder(connection)
                .path(conn_path.clone())
            {
                if let Ok(proxy) = conn_proxy.build().await {
                    if let Ok(settings) = proxy.get_settings().await {
                        if let Some(connection_settings) = settings.get("connection") {
                            if let Some(type_value) = connection_settings.get("type") {
                                if let Ok(conn_type) = type_value.downcast_ref::<zbus::zvariant::Str>()
                                {
                                    let conn_type_str = conn_type.as_str();

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

                                        let active = active_vpn_paths.contains(&conn_path.to_string());

                                        vpn_connections.push(VpnConnection {
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
        }

        Ok(vpn_connections)
    }
}
