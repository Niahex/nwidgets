use crate::services::bluetooth::{BluetoothService, BluetoothState};
use crate::theme::*;
use gpui::{div, prelude::*, rgb, Context};
use std::time::Duration;

pub struct BluetoothModule {
    state: BluetoothState,
}

impl BluetoothModule {
    pub fn new() -> Self {
        Self {
            state: BluetoothState {
                powered: false,
                connected_devices: 0,
            },
        }
    }

    pub fn update(&mut self, state: BluetoothState) {
        self.state = state;
    }

    pub fn get_state(&self) -> &BluetoothState {
        &self.state
    }

    /// Start monitoring Bluetooth - exposes the service's monitoring
    /// Panel will call this and listen to updates
    pub fn start_monitoring() -> std::sync::mpsc::Receiver<BluetoothState> {
        BluetoothService::start_monitoring()
    }

    /// Toggle Bluetooth power - to be called from event handlers
    pub async fn toggle_power() -> Result<bool, Box<dyn std::error::Error>> {
        BluetoothService::toggle_power().await.map_err(|e| e.into())
    }

    pub fn render<V: 'static>(&self, cx: &mut Context<V>) -> impl IntoElement {
        let (bt_icon, bt_color) = if !self.state.powered {
            (icons::BLUETOOTH_OFF, RED) // Off - red
        } else if self.state.connected_devices > 0 {
            (icons::BLUETOOTH_CONNECTED, FROST1) // Connected - blue
        } else {
            (icons::BLUETOOTH_ON, SNOW0) // On but not connected - white
        };

        let mut bt_widget = div()
            .w_12()
            .h_8()
            .rounded_md()
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(bt_color))
            .text_base()
            .cursor_pointer()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|_this, _event, _window, cx| {
                    cx.spawn(async move |_this, cx| {
                        match BluetoothModule::toggle_power().await {
                            Ok(new_state) => {
                                println!("[BLUETOOTH] 🔵 Toggled to: {}", new_state);
                            }
                            Err(e) => {
                                println!("[BLUETOOTH] ❌ Failed to toggle: {:?}", e);
                            }
                        }
                        let _ = cx;
                    })
                    .detach();
                }),
            )
            .child(bt_icon);

        // Show count if devices connected
        if self.state.connected_devices > 0 {
            bt_widget = bt_widget.child(
                div()
                    .text_xs()
                    .ml_0p5()
                    .child(format!("{}", self.state.connected_devices)),
            );
        }

        bt_widget
    }
}
