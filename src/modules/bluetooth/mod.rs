use zbus::{Connection, proxy};
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BluetoothState {
    pub powered: bool,
    pub connected_devices: usize,
}

// BlueZ Adapter interface
#[proxy(
    interface = "org.bluez.Adapter1",
    default_service = "org.bluez",
    default_path = "/org/bluez/hci0"
)]
trait Adapter {
    #[zbus(property)]
    fn powered(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_powered(&self, powered: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn discovering(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn discoverable(&self) -> zbus::Result<bool>;
}

// BlueZ Device interface
#[proxy(
    interface = "org.bluez.Device1",
    default_service = "org.bluez"
)]
trait Device {
    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn alias(&self) -> zbus::Result<String>;
}

pub struct BluetoothService;

impl BluetoothService {
    pub fn new() -> Self {
        Self
    }

    /// Start monitoring Bluetooth state changes
    pub fn start_monitoring() -> mpsc::Receiver<BluetoothState> {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    if let Ok(state) = Self::get_bluetooth_state().await {
                        let _ = tx.send(state);
                    }

                    // Poll every 5 seconds
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            });
        });

        rx
    }

    /// Get current Bluetooth state
    pub async fn get_bluetooth_state() -> zbus::Result<BluetoothState> {
        let connection = Connection::system().await?;

        // Get adapter state
        let adapter_proxy = AdapterProxy::new(&connection).await?;
        let powered = adapter_proxy.powered().await.unwrap_or(false);

        // Count connected devices
        let connected_devices = Self::count_connected_devices(&connection).await;

        Ok(BluetoothState {
            powered,
            connected_devices,
        })
    }

    async fn count_connected_devices(connection: &Connection) -> usize {
        // Get all objects from BlueZ
        let obj_manager = match zbus::fdo::ObjectManagerProxy::builder(connection)
            .destination("org.bluez")
            .ok()
            .and_then(|b| b.path("/").ok())
        {
            Some(builder) => match builder.build().await {
                Ok(om) => om,
                Err(_) => return 0,
            },
            None => return 0,
        };

        let objects = match obj_manager.get_managed_objects().await {
            Ok(objs) => objs,
            Err(_) => return 0,
        };

        let mut count = 0;
        for (path, interfaces) in objects {
            if interfaces.contains_key("org.bluez.Device1") {
                // Try to check if device is connected
                if let Some(builder) = DeviceProxy::builder(connection)
                    .path(path)
                    .ok()
                {
                    if let Ok(device_proxy) = builder.build().await {
                        if device_proxy.connected().await.unwrap_or(false) {
                            count += 1;
                        }
                    }
                }
            }
        }

        count
    }

    /// Toggle Bluetooth power state
    pub async fn toggle_power() -> zbus::Result<bool> {
        let connection = Connection::system().await?;
        let adapter_proxy = AdapterProxy::new(&connection).await?;

        let current = adapter_proxy.powered().await.unwrap_or(false);
        let new_state = !current;

        adapter_proxy.set_powered(new_state).await?;

        Ok(new_state)
    }
}
