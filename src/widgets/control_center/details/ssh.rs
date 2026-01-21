use crate::theme::ActiveTheme;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_ssh_details(
        &mut self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // TODO: Implémenter SSH details
        div().into_any_element()
    }
}
