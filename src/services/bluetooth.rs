use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    pub connected: bool,
    pub paired: bool,
    pub icon: String,
}

#[derive(Clone)]
pub struct BluetoothStateChanged {
    pub enabled: bool,
    pub devices: Vec<BluetoothDevice>,
}

pub struct BluetoothService {
    enabled: Arc<RwLock<bool>>,
    devices: Arc<RwLock<Vec<BluetoothDevice>>>,
}

impl EventEmitter<BluetoothStateChanged> for BluetoothService {}

impl BluetoothService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let enabled = Arc::new(RwLock::new(Self::fetch_bluetooth_state()));
        let devices = Arc::new(RwLock::new(Self::fetch_devices()));

        let enabled_clone = Arc::clone(&enabled);
        let devices_clone = Arc::clone(&devices);

        // Poll bluetooth state periodically (bluetoothctl doesn't have a great event API)
        cx.spawn(async move |this, mut cx| {
            Self::monitor_bluetooth(this, enabled_clone, devices_clone, &mut cx).await
        })
        .detach();

        Self { enabled, devices }
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    pub fn devices(&self) -> Vec<BluetoothDevice> {
        self.devices.read().clone()
    }

    pub fn toggle(&self) {
        let current = *self.enabled.read();
        std::thread::spawn(move || {
            let cmd = if current { "power off" } else { "power on" };
            let _ = std::process::Command::new("bluetoothctl")
                .args([cmd])
                .status();
        });
    }

    pub fn connect_device(&self, address: String) {
        std::thread::spawn(move || {
            let _ = std::process::Command::new("bluetoothctl")
                .args(["connect", &address])
                .status();
        });
    }

    pub fn disconnect_device(&self, address: String) {
        std::thread::spawn(move || {
            let _ = std::process::Command::new("bluetoothctl")
                .args(["disconnect", &address])
                .status();
        });
    }

    async fn monitor_bluetooth(
        this: WeakEntity<Self>,
        enabled: Arc<RwLock<bool>>,
        devices: Arc<RwLock<Vec<BluetoothDevice>>>,
        cx: &mut AsyncApp,
    ) {
        loop {
            // Poll every 2 seconds
            cx.background_executor()
                .timer(Duration::from_secs(2))
                .await;

            let new_enabled = Self::fetch_bluetooth_state();
            let new_devices = Self::fetch_devices();

            let state_changed = {
                let mut current_enabled = enabled.write();
                let mut current_devices = devices.write();
                let changed = *current_enabled != new_enabled || *current_devices != new_devices;
                if changed {
                    *current_enabled = new_enabled;
                    *current_devices = new_devices.clone();
                }
                changed
            };

            if state_changed {
                if let Ok(()) = this.update(cx, |_, cx| {
                    cx.emit(BluetoothStateChanged {
                        enabled: new_enabled,
                        devices: new_devices,
                    });
                    cx.notify();
                }) {}
            }
        }
    }

    fn fetch_bluetooth_state() -> bool {
        let output = std::process::Command::new("bluetoothctl")
            .args(["show"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        output.lines().any(|line| line.contains("Powered: yes"))
    }

    fn fetch_devices() -> Vec<BluetoothDevice> {
        let output = std::process::Command::new("bluetoothctl")
            .args(["devices"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        // Simple parsing - you might want to improve this
        output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "Device" {
                    let address = parts[1].to_string();
                    let name = parts[2..].join(" ");
                    Some(BluetoothDevice {
                        name,
                        address,
                        connected: false, // TODO: Check connected status
                        paired: true,
                        icon: "bluetooth".to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

// Global accessor
struct GlobalBluetoothService(Entity<BluetoothService>);
impl Global for GlobalBluetoothService {}

impl BluetoothService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalBluetoothService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|cx| Self::new(cx));
        cx.set_global(GlobalBluetoothService(service.clone()));
        service
    }
}
