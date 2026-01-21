use crate::components::{Dropdown, DropdownOption};
use crate::services::audio::AudioService;
use crate::theme::ActiveTheme;
use crate::utils::Icon;
use crate::widgets::control_center::get_stream_display;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_source_details(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // TODO: Implémenter source details
        div().into_any_element()
    }
}
