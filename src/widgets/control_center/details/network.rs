use crate::services::network::NetworkService;
use crate::theme::ActiveTheme;
use crate::components::Toggle;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_network_details(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // TODO: Implémenter
        div().into_any_element()
    }
}
