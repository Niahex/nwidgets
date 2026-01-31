use std::sync::Arc;
use parking_lot::RwLock;
use zbus::{Connection, proxy};

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug, Default)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub alias: String,
    pub is_connected: bool,
    pub is_paired: bool,
    pub is_trusted: bool,
    pub icon: String,
}

#[derive(Clone)]
pub struct BluetoothService {
    state: Arc<RwLock<BluetoothState>>,
}

#[derive(Default)]
struct BluetoothState {
    is_powered: bool,
    is_discovering: bool,
    devices: Vec<BluetoothDevice>,
}

impl BluetoothService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(BluetoothState::default())),
        };

        service.start();
        service
    }

    fn start(&self) {
        let state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            if let Err(e) = Self::monitor_bluetooth(state).await {
                log::error!("Bluetooth monitor error: {}", e);
            }
        });
    }

    async fn monitor_bluetooth(state: Arc<RwLock<BluetoothState>>) -> anyhow::Result<()> {
        let connection = Connection::system().await?;

        let managed_objects = Self::get_managed_objects(&connection).await?;
        Self::process_objects(&state, managed_objects);

        Ok(())
    }

    async fn get_managed_objects(connection: &Connection) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }

    fn process_objects(_state: &Arc<RwLock<BluetoothState>>, _objects: Vec<String>) {
    }

    pub fn is_powered(&self) -> bool {
        self.state.read().is_powered
    }

    pub fn get_devices(&self) -> Vec<BluetoothDevice> {
        self.state.read().devices.clone()
    }

    pub fn get_connected_devices(&self) -> Vec<BluetoothDevice> {
        self.state.read().devices.iter()
            .filter(|d| d.is_connected)
            .cloned()
            .collect()
    }
}
