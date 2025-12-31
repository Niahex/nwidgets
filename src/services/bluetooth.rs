use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BluetoothState {
    pub powered: bool,
    pub connected_devices: usize,
}


#[derive(Clone)]
pub struct BluetoothStateChanged {
    pub state: BluetoothState,
}

pub struct BluetoothService {
    state: Arc<RwLock<BluetoothState>>,
}

impl EventEmitter<BluetoothStateChanged> for BluetoothService {}

impl BluetoothService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = Arc::new(RwLock::new(Self::fetch_bluetooth_state()));
        let state_clone = Arc::clone(&state);

        cx.spawn(async move |this, cx| {
            Self::monitor_bluetooth(this, state_clone, cx).await
        })
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
        loop {
            cx.background_executor().timer(Duration::from_secs(2)).await;

            let new_state = Self::fetch_bluetooth_state();

            let state_changed = {
                let mut current_state = state.write();
                let changed = *current_state != new_state;
                if changed {
                    *current_state = new_state.clone();
                }
                changed
            };

            if state_changed {
                let _ = this.update(cx, |_, cx| {
                    cx.emit(BluetoothStateChanged { state: new_state });
                    cx.notify();
                });
            }
        }
    }

    fn fetch_bluetooth_state() -> BluetoothState {
        let output = Command::new("bluetoothctl")
            .args(["show"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let powered = output.contains("Powered: yes");

        let devices_output = Command::new("bluetoothctl")
            .args(["devices", "Connected"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let connected_devices = devices_output.lines().count();

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
