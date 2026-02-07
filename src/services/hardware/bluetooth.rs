use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use zbus::{
    proxy,
    zvariant::{OwnedObjectPath, OwnedValue},
    Connection, Result,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BluetoothDevice {
    pub name: SharedString,
    pub address: String,
    pub connected: bool,
    pub auto_connect: bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BluetoothState {
    pub powered: bool,
    pub connected_devices: usize,
    pub devices: Vec<BluetoothDevice>,
}

#[derive(Clone)]
pub struct BluetoothStateChanged;

pub struct BluetoothService {
    state: Arc<RwLock<BluetoothState>>,
}

impl EventEmitter<BluetoothStateChanged> for BluetoothService {}

type ManagedObjects = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;

// --- DBus Interfaces ---

#[proxy(
    default_service = "org.bluez",
    default_path = "/",
    interface = "org.freedesktop.DBus.ObjectManager"
)]
trait ObjectManager {
    fn get_managed_objects(&self) -> Result<ManagedObjects>;

    #[zbus(signal)]
    fn interfaces_added(
        &self,
        object_path: OwnedObjectPath,
        interfaces_and_properties: HashMap<String, HashMap<String, OwnedValue>>,
    ) -> Result<()>;

    #[zbus(signal)]
    fn interfaces_removed(
        &self,
        object_path: OwnedObjectPath,
        interfaces: Vec<String>,
    ) -> Result<()>;
}

// --- Service Implementation ---

impl BluetoothService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Initial state is default until first DBus fetch
        let state = Arc::new(RwLock::new(BluetoothState::default()));
        let state_clone = Arc::clone(&state);

        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<BluetoothState>();

        // 1. Worker Task (Tokio)
        gpui_tokio::Tokio::spawn(cx, async move { Self::bluetooth_worker(ui_tx).await }).detach();

        // 2. UI Task (GPUI)
        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(new_state) = ui_rx.next().await {
                    let state_changed = {
                        let mut current_state = state_clone.write();
                        if *current_state != new_state {
                            *current_state = new_state;
                            true
                        } else {
                            false
                        }
                    };

                    if state_changed {
                        let _ = this.update(&mut cx, |_, cx| {
                            cx.emit(BluetoothStateChanged);
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();

        Self { state }
    }

    pub fn state(&self) -> BluetoothState {
        self.state.read().clone()
    }

    async fn bluetooth_worker(ui_tx: futures::channel::mpsc::UnboundedSender<BluetoothState>) {
        let conn = match Connection::system().await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to connect to system bus: {e}");
                return;
            }
        };

        let object_manager = match ObjectManagerProxy::new(&conn).await {
            Ok(p) => p,
            Err(e) => {
                log::error!("Failed to create ObjectManager proxy: {e}");
                return;
            }
        };

        // Initial fetch
        let initial_state = Self::fetch_bluetooth_state_dbus(&object_manager).await;
        let _ = ui_tx.unbounded_send(initial_state);

        // Get event streams
        let mut added_stream = match object_manager.receive_interfaces_added().await {
            Ok(s) => Some(s),
            Err(e) => {
                log::error!("Failed to receive InterfacesAdded signal: {e}");
                None
            }
        };

        let mut removed_stream = match object_manager.receive_interfaces_removed().await {
            Ok(s) => Some(s),
            Err(e) => {
                log::error!("Failed to receive InterfacesRemoved signal: {e}");
                None
            }
        };

        loop {
            tokio::select! {
                Some(_) = async {
                    if let Some(s) = &mut added_stream { s.next().await } else { std::future::pending().await }
                } => {
                    let new_state = Self::fetch_bluetooth_state_dbus(&object_manager).await;
                    let _ = ui_tx.unbounded_send(new_state);
                }
                Some(_) = async {
                    if let Some(s) = &mut removed_stream { s.next().await } else { std::future::pending().await }
                } => {
                    let new_state = Self::fetch_bluetooth_state_dbus(&object_manager).await;
                    let _ = ui_tx.unbounded_send(new_state);
                }
                _ = tokio::time::sleep(Duration::from_secs(2)) => {
                    let new_state = Self::fetch_bluetooth_state_dbus(&object_manager).await;
                    let _ = ui_tx.unbounded_send(new_state);
                }
            }
        }
    }

    async fn fetch_bluetooth_state_dbus(om: &ObjectManagerProxy<'_>) -> BluetoothState {
        let mut powered = false;
        let mut connected_devices = 0;
        let mut devices = Vec::with_capacity(10);

        if let Ok(objects) = om.get_managed_objects().await {
            for (_path, interfaces) in objects {
                // Check Adapter interface for Powered state
                if let Some(adapter) = interfaces.get("org.bluez.Adapter1") {
                    if let Some(value) = adapter.get("Powered") {
                        if let Ok(p) = bool::try_from(value) {
                            if p {
                                powered = true;
                            }
                        }
                    }
                }

                // Check Device interface for Connected state
                if let Some(device) = interfaces.get("org.bluez.Device1") {
                    let mut name = String::new();
                    let mut address = String::new();
                    let mut connected = false;

                    if let Some(value) = device.get("Name") {
                        if let Ok(n) = <&str>::try_from(value) {
                            name = n.to_string();
                        }
                    }
                    if let Some(value) = device.get("Address") {
                        if let Ok(a) = <&str>::try_from(value) {
                            address = a.to_string();
                        }
                    }
                    if let Some(value) = device.get("Connected") {
                        if let Ok(c) = bool::try_from(value) {
                            connected = c;
                            if connected {
                                connected_devices += 1;
                            }
                        }
                    }

                    let auto_connect = device
                        .get("Trusted")
                        .and_then(|v| bool::try_from(v).ok())
                        .unwrap_or(false);

                    if !address.is_empty() {
                        devices.push(BluetoothDevice {
                            name: if name.is_empty() {
                                address.clone().into()
                            } else {
                                name.into()
                            },
                            address,
                            connected,
                            auto_connect,
                        });
                    }
                }
            }
        }

        BluetoothState {
            powered,
            connected_devices,
            devices,
        }
    }
}

struct GlobalBluetoothService(Entity<BluetoothService>);
impl Global for GlobalBluetoothService {}

#[proxy(default_service = "org.bluez", interface = "org.bluez.Adapter1")]
trait Adapter {
    #[zbus(property)]
    fn powered(&self) -> Result<bool>;

    #[zbus(property)]
    fn set_powered(&self, value: bool) -> Result<()>;
}

#[proxy(default_service = "org.bluez", interface = "org.bluez.Device1")]
trait Device {
    fn connect(&self) -> Result<()>;
    fn disconnect(&self) -> Result<()>;

    #[zbus(property)]
    fn trusted(&self) -> Result<bool>;

    #[zbus(property)]
    fn set_trusted(&self, value: bool) -> Result<()>;
}

impl BluetoothService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalBluetoothService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalBluetoothService(service.clone()));
        service
    }

    pub fn toggle_power(&self, cx: &mut Context<Self>) {
        let current_powered = self.state.read().powered;
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Err(e) = Self::set_adapter_power(!current_powered).await {
                log::error!("Failed to toggle bluetooth power: {e}");
            }
        })
        .detach();
    }

    pub fn toggle_device(&self, address: String, cx: &mut Context<Self>) {
        let powered = self.state.read().powered;
        gpui_tokio::Tokio::spawn(cx, async move {
            // Enable bluetooth first if it's off
            if !powered {
                if let Err(e) = Self::set_adapter_power(true).await {
                    log::error!("Failed to enable bluetooth: {e}");
                    return;
                }
                // Wait a bit for bluetooth to be ready
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }

            if let Err(e) = Self::toggle_device_connection(&address).await {
                log::error!("Failed to toggle device connection: {e}");
            }
        })
        .detach();
    }

    pub fn toggle_auto_connect(&self, address: String, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Err(e) = Self::toggle_device_trusted(&address).await {
                log::error!("Failed to toggle auto-connect: {e}");
            }
        })
        .detach();
    }

    async fn set_adapter_power(powered: bool) -> Result<()> {
        let conn = Connection::system().await?;
        let om = ObjectManagerProxy::new(&conn).await?;
        let objects = om.get_managed_objects().await?;

        for (path, interfaces) in objects {
            if interfaces.contains_key("org.bluez.Adapter1") {
                let adapter = AdapterProxy::builder(&conn).path(path)?.build().await?;
                adapter.set_powered(powered).await?;
                return Ok(());
            }
        }
        Ok(())
    }

    async fn toggle_device_connection(address: &str) -> Result<()> {
        let conn = Connection::system().await?;
        let om = ObjectManagerProxy::new(&conn).await?;
        let objects = om.get_managed_objects().await?;

        for (path, interfaces) in objects {
            if let Some(device) = interfaces.get("org.bluez.Device1") {
                if let Some(addr_value) = device.get("Address") {
                    if let Ok(addr) = <&str>::try_from(addr_value) {
                        if addr == address {
                            let device_proxy =
                                DeviceProxy::builder(&conn).path(path)?.build().await?;

                            if let Some(connected_value) = device.get("Connected") {
                                if let Ok(connected) = bool::try_from(connected_value) {
                                    if connected {
                                        device_proxy.disconnect().await?;
                                    } else {
                                        device_proxy.connect().await?;
                                    }
                                }
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn toggle_device_trusted(address: &str) -> Result<()> {
        let conn = Connection::system().await?;
        let om = ObjectManagerProxy::new(&conn).await?;
        let objects = om.get_managed_objects().await?;

        for (path, interfaces) in objects {
            if let Some(device) = interfaces.get("org.bluez.Device1") {
                if let Some(addr_value) = device.get("Address") {
                    if let Ok(addr) = <&str>::try_from(addr_value) {
                        if addr == address {
                            let device_proxy =
                                DeviceProxy::builder(&conn).path(path)?.build().await?;

                            let current_trusted = device_proxy.trusted().await.unwrap_or(false);
                            device_proxy.set_trusted(!current_trusted).await?;
                            return Ok(());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
