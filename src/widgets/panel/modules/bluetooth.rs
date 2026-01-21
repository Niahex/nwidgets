use crate::services::bluetooth::{BluetoothService, BluetoothStateChanged};
use crate::assets::Icon;
use gpui::prelude::*;
use gpui::*;

pub struct BluetoothModule {
    bluetooth: Entity<BluetoothService>,
}

impl BluetoothModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let bluetooth = BluetoothService::global(cx);

        cx.subscribe(
            &bluetooth,
            |_this, _bluetooth, _event: &BluetoothStateChanged, cx| {
                cx.notify();
            },
        )
        .detach();

        Self { bluetooth }
    }
}

impl Render for BluetoothModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.bluetooth.read(cx).state();

        let icon_name = if !state.powered {
            "bluetooth-disabled"
        } else if state.connected_devices > 0 {
            "bluetooth-active"
        } else {
            "bluetooth-paired"
        };

        Icon::new(icon_name).size(px(16.)).preserve_colors(true)
    }
}
