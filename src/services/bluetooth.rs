use zbus::{Connection, proxy};
use std::sync::mpsc;
use glib::MainContext;

#[derive(Debug, Clone)]
pub struct BluetoothState {
    pub powered: bool,
    pub connected_devices: usize,
}

#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub connected: bool,
    pub paired: bool,
    pub path: String,
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

    #[zbus(property)]
    fn address(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn paired(&self) -> zbus::Result<bool>;

    fn connect(&self) -> zbus::Result<()>;

    fn disconnect(&self) -> zbus::Result<()>;

    fn pair(&self) -> zbus::Result<()>;
}

pub struct BluetoothService;

impl BluetoothService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// Abonne un callback aux changements d'état Bluetooth
    /// Le callback sera appelé sur le thread principal GTK
    pub fn subscribe_bluetooth<F>(callback: F)
    where
        F: Fn(BluetoothState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        // Thread qui monitore le bluetooth
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    if let Ok(state) = Self::get_bluetooth_state().await {
                        if tx.send(state).is_err() {
                            break;
                        }
                    }

                    // Poll every 5 seconds
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            });
        });

        // Créer un async channel pour exécuter le callback sur le thread principal
        let (async_tx, async_rx) = async_channel::unbounded();

        // Thread qui reçoit les mises à jour et les transfère au async channel
        std::thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                if async_tx.send_blocking(state).is_err() {
                    break;
                }
            }
        });

        // Attacher le callback au async channel
        MainContext::default().spawn_local(async move {
            while let Ok(state) = async_rx.recv().await {
                callback(state);
            }
        });
    }

    /// Start monitoring Bluetooth state changes (ancienne méthode conservée pour compatibilité)
    #[allow(dead_code)]
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

    /// List all Bluetooth devices (paired and available)
    pub fn list_devices() -> Vec<BluetoothDevice> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            Self::list_devices_async().await.unwrap_or_default()
        })
    }

    async fn list_devices_async() -> zbus::Result<Vec<BluetoothDevice>> {
        let connection = Connection::system().await?;
        let obj_manager = zbus::fdo::ObjectManagerProxy::builder(&connection)
            .destination("org.bluez")?
            .path("/")?
            .build()
            .await?;

        let objects = obj_manager.get_managed_objects().await?;
        let mut devices = Vec::new();

        for (path, interfaces) in objects {
            if interfaces.contains_key("org.bluez.Device1") {
                if let Ok(device_proxy) = DeviceProxy::builder(&connection)
                    .path(path.clone())?
                    .build()
                    .await
                {
                    let name = match device_proxy.alias().await {
                        Ok(alias) => alias,
                        Err(_) => device_proxy.name().await.unwrap_or_else(|_| "Unknown Device".to_string()),
                    };

                    let address = device_proxy.address().await.unwrap_or_default();
                    let connected = device_proxy.connected().await.unwrap_or(false);
                    let paired = device_proxy.paired().await.unwrap_or(false);

                    devices.push(BluetoothDevice {
                        address,
                        name,
                        connected,
                        paired,
                        path: path.to_string(),
                    });
                }
            }
        }

        Ok(devices)
    }

    /// Connect to a Bluetooth device
    pub fn connect_device(device_path: &str) {
        let path = device_path.to_string();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = Self::connect_device_async(&path).await {
                    eprintln!("Failed to connect device: {}", e);
                }
            });
        });
    }

    async fn connect_device_async(device_path: &str) -> zbus::Result<()> {
        let connection = Connection::system().await?;
        let device_proxy = DeviceProxy::builder(&connection)
            .path(device_path)?
            .build()
            .await?;

        device_proxy.connect().await?;
        Ok(())
    }

    /// Disconnect a Bluetooth device
    pub fn disconnect_device(device_path: &str) {
        let path = device_path.to_string();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = Self::disconnect_device_async(&path).await {
                    eprintln!("Failed to disconnect device: {}", e);
                }
            });
        });
    }

    async fn disconnect_device_async(device_path: &str) -> zbus::Result<()> {
        let connection = Connection::system().await?;
        let device_proxy = DeviceProxy::builder(&connection)
            .path(device_path)?
            .build()
            .await?;

        device_proxy.disconnect().await?;
        Ok(())
    }
}
