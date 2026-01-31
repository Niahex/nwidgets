use std::sync::Arc;
use parking_lot::RwLock;
use zbus::Connection;

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug, Default)]
pub struct NetworkState {
    pub wifi_connected: bool,
    pub wifi_ssid: Option<String>,
    pub wifi_strength: u8,
    pub ethernet_connected: bool,
    pub vpn_connected: bool,
    pub vpn_name: Option<String>,
}

#[derive(Clone)]
pub struct NetworkService {
    state: Arc<RwLock<NetworkState>>,
}

impl NetworkService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(NetworkState::default())),
        };

        service.start();
        service
    }

    fn start(&self) {
        let state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::monitor_network(state).await {
                log::error!("Network monitor error: {}", e);
            }
        });
    }

    async fn monitor_network(_state: Arc<RwLock<NetworkState>>) -> anyhow::Result<()> {
        let _connection = Connection::system().await?;

        log::info!("Network service started");

        Ok(())
    }

    pub fn get_state(&self) -> NetworkState {
        self.state.read().clone()
    }

    pub fn is_connected(&self) -> bool {
        let state = self.state.read();
        state.wifi_connected || state.ethernet_connected
    }
}
