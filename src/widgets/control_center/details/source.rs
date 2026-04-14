use super::audio_device::AudioDeviceType;
use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_source_details(
        &mut self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let sources = self.audio.read(cx).sources();
        let streams = self.audio.read(cx).source_outputs();
        let is_open = self.source_dropdown_open;

        self.render_audio_device_panel(
            AudioDeviceType::Source,
            sources,
            streams,
            is_open,
            |this, cx| {
                this.source_dropdown_open = !this.source_dropdown_open;
                cx.notify();
            },
            |this, id, cx| {
                this.audio.update(cx, |audio, cx| {
                    audio.set_default_source(id, cx);
                });
                this.source_dropdown_open = false;
                cx.notify();
            },
            theme,
            cx,
        )
    }
}
