use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    BluetoothDeviceRow = <View> {
        width: Fill, height: 48
        flow: Right
        align: {x: 0.0, y: 0.5}
        padding: {left: 12, right: 12}
        spacing: 12

        icon = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 16.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
            text: "󰋋"
        }

        info = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 2

            name = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 12.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
                text: "Device Name"
            }

            status = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 10.0 }, color: (THEME_COLOR_TEXT_MUTE) }
                text: "Connected"
            }
        }
    }

    pub BluetoothDetails = {{BluetoothDetails}} {
        width: Fill, height: Fill
        flow: Down
        padding: 16
        spacing: 12

        header = <View> {
            width: Fill, height: Fit
            flow: Row
            align: {x: 0.0, y: 0.5}

            title = <Label> {
                draw_text: { text_style: <THEME_FONT_BOLD> { font_size: 14.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
                text: "Bluetooth Devices"
            }
        }

        devices_list = <PortalList> {
            width: Fill, height: Fill

            BluetoothDeviceRow = <BluetoothDeviceRow> {}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct BluetoothDetails {
    #[deref]
    view: View,

    #[rust]
    devices: Vec<BluetoothDevice>,
}

#[derive(Clone, Debug)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub is_connected: bool,
    pub is_trusted: bool,
    pub device_type: BluetoothDeviceType,
}

#[derive(Clone, Debug)]
pub enum BluetoothDeviceType {
    Headphones,
    Keyboard,
    Mouse,
    Phone,
    Other,
}

impl Widget for BluetoothDetails {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.devices.len());

                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < self.devices.len() {
                        let device = &self.devices[item_id];
                        let item = list.item(cx, item_id, live_id!(BluetoothDeviceRow));

                        item.label(ids!(info.name)).set_text(cx, &device.name);

                        let status = if device.is_connected { "Connected" } else { "Not connected" };
                        item.label(ids!(info.status)).set_text(cx, status);

                        let icon = match device.device_type {
                            BluetoothDeviceType::Headphones => "󰋋",
                            BluetoothDeviceType::Keyboard => "󰌌",
                            BluetoothDeviceType::Mouse => "󰍽",
                            BluetoothDeviceType::Phone => "󰏲",
                            BluetoothDeviceType::Other => "󰂯",
                        };
                        item.label(ids!(icon)).set_text(cx, icon);

                        item.draw_all(cx, &mut Scope::empty());
                    }
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl BluetoothDetails {
    pub fn set_devices(&mut self, cx: &mut Cx, devices: Vec<BluetoothDevice>) {
        self.devices = devices;
        self.view.redraw(cx);
    }
}
