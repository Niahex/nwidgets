use super::audio_device::AudioDeviceType;
use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_sink_details(
        &mut self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let sinks = self.audio.read(cx).sinks();
        let streams = self.audio.read(cx).sink_inputs();
        let is_open = self.sink_dropdown_open;

        self.render_audio_device_panel(
            AudioDeviceType::Sink,
            sinks,
            streams,
            is_open,
            |this, cx| {
                this.sink_dropdown_open = !this.sink_dropdown_open;
                cx.notify();
            },
            |this, id, cx| {
                this.audio.update(cx, |audio, cx| {
                    audio.set_default_sink(id, cx);
                });
                this.sink_dropdown_open = false;
                cx.notify();
            },
            theme,
            cx,
        )
    }
}
