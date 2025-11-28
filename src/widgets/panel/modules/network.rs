use super::base::{PanelModuleConfig, update_icon};
use crate::services::network::NetworkState;
use gtk4 as gtk;

#[derive(Clone)]
pub struct NetworkModule {
    pub container: gtk::CenterBox,
    icon: gtk::Image,
}

impl NetworkModule {
    pub fn new() -> Self {
        let config = PanelModuleConfig::new(
            "network-widget",
            "network-icon",
            "network-disconnected",
            "network",
        );
        let (container, icon) = config.build();
        Self { container, icon }
    }

    pub fn update(&self, state: NetworkState) {
        let icon_name = state.get_icon_name();
        update_icon(&self.icon, icon_name, Some(20));
    }
}
