use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use zbus::{
    proxy,
    zvariant::{OwnedObjectPath, OwnedValue},
    Connection, Result,
};
use futures::StreamExt;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BluetoothState {
    pub powered: bool,
    pub connected_devices: usize,
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
        gpui_tokio::Tokio::spawn(cx, async move {
            Self::bluetooth_worker(ui_tx).await
        })
        .detach();

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
                eprintln!("[BluetoothService] Failed to connect to system bus: {e}");
                return;
            }
        };

        let object_manager = match ObjectManagerProxy::new(&conn).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[BluetoothService] Failed to create ObjectManager proxy: {e}");
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
                eprintln!("[BluetoothService] Failed to receive InterfacesAdded signal: {e}");
                None
            }
        };

        let mut removed_stream = match object_manager.receive_interfaces_removed().await {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("[BluetoothService] Failed to receive InterfacesRemoved signal: {e}");
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
                    if let Some(value) = device.get("Connected") {
                        if let Ok(connected) = bool::try_from(value) {
                            if connected {
                                connected_devices += 1;
                            }
                        }
                    }
                }
            }
        }

        BluetoothState {
            powered,
            connected_devices,
        }
    }
}

struct GlobalBluetoothService(Entity<BluetoothService>);
impl Global for GlobalBluetoothService {}

impl BluetoothService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalBluetoothService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalBluetoothService(service.clone()));
        service
    }
}