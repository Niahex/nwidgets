use crate::components::Toggle;
use crate::services::bluetooth::BluetoothService;
use crate::services::control_center::ControlCenterSection;
use crate::services::network::NetworkService;
use crate::services::system_monitor::SystemMonitorService;
use crate::theme::ActiveTheme;
use crate::utils::Icon;
use gpui::*;

impl super::ControlCenterWidget {
    pub(super) fn render_connectivity_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // TODO: Implémenter quick actions (monitor, bluetooth, network, proxy, ssh, vm)
        div()
    }
}
