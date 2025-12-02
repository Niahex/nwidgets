use super::base::{update_icon, PanelModuleConfig};
use crate::services::bluetooth::BluetoothState;
use gtk4 as gtk;

#[derive(Clone)]
pub struct BluetoothModule {
    pub container: gtk::CenterBox,
    icon: gtk::Image,
}

impl BluetoothModule {
    pub fn new() -> Self {
        let config = PanelModuleConfig::new(
            "bluetooth-widget",
            "bluetooth-icon",
            "bluetooth-disabled",
            "bluetooth",
        );
        let (container, icon) = config.build();
        Self { container, icon }
    }

    pub fn update(&self, state: BluetoothState) {
        let icon_name = if !state.powered {
            "bluetooth-disabled"
        } else if state.connected_devices > 0 {
            "bluetooth-active"
        } else {
            "bluetooth-paired"
        };

        update_icon(&self.icon, icon_name, Some(20));
    }
}
