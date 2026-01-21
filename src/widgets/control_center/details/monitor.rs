use crate::components::CircularProgress;
use crate::services::system_monitor::SystemMonitorService;
use crate::theme::ActiveTheme;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_monitor_details(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // TODO: Implémenter
        div().into_any_element()
    }
}
