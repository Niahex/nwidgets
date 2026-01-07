use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use zbus::{proxy, zvariant::{OwnedObjectPath, OwnedValue}, Connection, Result};

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
}

// --- Service Implementation ---

impl BluetoothService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Initial state is default until first DBus fetch
        let state = Arc::new(RwLock::new(BluetoothState::default()));
        let state_clone = Arc::clone(&state);

        cx.spawn(async move |this, cx| Self::monitor_bluetooth(this, state_clone, cx).await)
            .detach();

        Self { state }
    }

    pub fn state(&self) -> BluetoothState {
        self.state.read().clone()
    }

    async fn monitor_bluetooth(
        this: WeakEntity<Self>,
        state: Arc<RwLock<BluetoothState>>,
        cx: &mut AsyncApp,
    ) {
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

        loop {
            let new_state = Self::fetch_bluetooth_state_dbus(&object_manager).await;

            let state_changed = {
                let mut current_state = state.write();
                if *current_state != new_state {
                    *current_state = new_state;
                    true
                } else {
                    false
                }
            };

            if state_changed {
                let _ = this.update(cx, |_, cx| {
                    cx.emit(BluetoothStateChanged);
                    cx.notify();
                });
            }

            // Polling via DBus is cheap, 2 seconds is fine.
            cx.background_executor().timer(Duration::from_secs(2)).await;
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
