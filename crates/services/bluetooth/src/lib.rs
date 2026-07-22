use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global};
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BluetoothState {
    pub powered: bool,
    pub devices: Vec<BluetoothDevice>,
}

#[derive(Debug, Clone)]
pub struct BluetoothStateChanged;

pub struct BluetoothService {
    pub state: BluetoothState,
}

impl EventEmitter<BluetoothStateChanged> for BluetoothService {}

struct GlobalBluetoothService(Entity<BluetoothService>);
impl Global for GlobalBluetoothService {}

impl BluetoothService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalBluetoothService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|_cx| Self {
            state: BluetoothState::default(),
        });

        cx.set_global(GlobalBluetoothService(service.clone()));

        let (tx, mut rx) = mpsc::unbounded::<BluetoothState>();

        // Background worker to query bluetoothctl
        gpui_tokio::Tokio::spawn(cx, async move {
            let mut state = BluetoothState::default();

            if let Ok(out) = Command::new("bluetoothctl").arg("show").output().await {
                let s = String::from_utf8_lossy(&out.stdout);
                state.powered = s.contains("Powered: yes");
            }

            if let Ok(out) = Command::new("bluetoothctl").arg("devices").output().await {
                let s = String::from_utf8_lossy(&out.stdout);
                for line in s.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 && parts[0] == "Device" {
                        let address = parts[1].to_string();
                        let name = parts[2..].join(" ");
                        state.devices.push(BluetoothDevice {
                            name,
                            address,
                            connected: false,
                        });
                    }
                }
            }

            let _ = tx.unbounded_send(state);
        })
        .detach();

        // UI Thread listener
        let service_entity = service.clone();
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(new_state) = rx.next().await {
                    let _ = cx.update(|cx| {
                        service_entity.update(cx, |srv, cx| {
                            if srv.state != new_state {
                                srv.state = new_state;
                                cx.emit(BluetoothStateChanged);
                                cx.notify();
                            }
                        });
                    });
                }
            }
        })
        .detach();

        service
    }

    pub fn toggle_power(&mut self, cx: &mut Context<Self>) {
        self.state.powered = !self.state.powered;
        cx.notify();
        let target_state = if self.state.powered { "on" } else { "off" };
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = Command::new("bluetoothctl")
                .args(["power", target_state])
                .status()
                .await;
        })
        .detach();
    }
}
