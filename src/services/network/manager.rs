use futures::StreamExt;
use parking_lot::RwLock;
use std::sync::Arc;
use zbus::{proxy, zvariant::OwnedObjectPath, Connection};

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionType {
    Wifi,
    Ethernet,
    Vpn,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveConnectionInfo {
    pub id: String,
    pub conn_type: ConnectionType,
    pub specific_object: OwnedObjectPath,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NetworkManagerState {
    pub connected: bool,
    pub active_connections: Vec<ActiveConnectionInfo>,
}

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
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
pub trait AccessPoint {
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;
}

pub struct NetworkManagerWorker {
    state: Arc<RwLock<NetworkManagerState>>,
}

impl NetworkManagerWorker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(NetworkManagerState::default())),
        }
    }

    pub async fn run(&self, tx: futures::channel::mpsc::UnboundedSender<NetworkManagerState>) {
        let timeout = std::time::Duration::from_secs(3);
        
        let conn = match tokio::time::timeout(timeout, Connection::system()).await {
            Ok(Ok(c)) => c,
            Ok(Err(_)) | Err(_) => return,
        };

        let nm_proxy = match tokio::time::timeout(timeout, NetworkManagerProxy::new(&conn)).await {
            Ok(Ok(p)) => p,
            Ok(Err(_)) | Err(_) => return,
        };

        let initial_state = Self::fetch_state(&conn, &nm_proxy).await;
        if let Err(e) = tx.unbounded_send(initial_state.clone()) {
            log::warn!("Failed to send initial network manager state: {}", e);
        }
        *self.state.write() = initial_state;

        let mut connectivity_stream = nm_proxy.receive_connectivity_changed().await;
        let mut active_connections_stream = nm_proxy.receive_active_connections_changed().await;

        loop {
            tokio::select! {
                Some(_) = connectivity_stream.next() => {
                    let new_state = Self::fetch_state(&conn, &nm_proxy).await;
                    if *self.state.read() != new_state {
                        *self.state.write() = new_state.clone();
                        if let Err(e) = tx.unbounded_send(new_state) {
                            log::warn!("Failed to send network manager connectivity update: {}", e);
                        }
                    }
                }
                Some(_) = active_connections_stream.next() => {
                    let new_state = Self::fetch_state(&conn, &nm_proxy).await;
                    if *self.state.read() != new_state {
                        *self.state.write() = new_state.clone();
                        if let Err(e) = tx.unbounded_send(new_state) {
                            log::warn!("Failed to send network manager connections update: {}", e);
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                    let new_state = Self::fetch_state(&conn, &nm_proxy).await;
                    if *self.state.read() != new_state {
                        *self.state.write() = new_state.clone();
                        if let Err(e) = tx.unbounded_send(new_state) {
                            log::warn!("Failed to send network manager polling update: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn fetch_state(conn: &Connection, nm: &NetworkManagerProxy<'_>) -> NetworkManagerState {
        let mut state = NetworkManagerState::default();

        if let Ok(connectivity) = nm.connectivity().await {
            state.connected = connectivity >= 4;
        }

        if let Ok(active_paths) = nm.active_connections().await {
            for path in active_paths {
                if let Ok(ac) = ActiveConnectionProxy::new(conn, path.clone()).await {
                    if let (Ok(id), Ok(type_str), Ok(specific_object)) =
                        (ac.id().await, ac.type_().await, ac.specific_object().await)
                    {
                        let conn_type = match type_str.as_str() {
                            "802-11-wireless" => Some(ConnectionType::Wifi),
                            "802-3-ethernet" => Some(ConnectionType::Ethernet),
                            "vpn" | "wireguard" => Some(ConnectionType::Vpn),
                            _ => {
                                if let Ok(is_vpn) = ac.vpn().await {
                                    if is_vpn {
                                        Some(ConnectionType::Vpn)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        };

                        if let Some(conn_type) = conn_type {
                            state.active_connections.push(ActiveConnectionInfo {
                                id,
                                conn_type,
                                specific_object,
                            });
                        }
                    }
                }
            }
        }

        state
    }
}
