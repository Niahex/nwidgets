use crate::components::{Dropdown, DropdownOption};
use crate::services::media::audio::{AudioDevice, AudioStream};
use crate::theme::Theme;
use gpui::prelude::*;
use gpui::*;
use smallvec::SmallVec;

pub enum AudioDeviceType {
    Sink,
    Source,
}

impl AudioDeviceType {
    fn dropdown_id(&self) -> &'static str {
        match self {
            AudioDeviceType::Sink => "sink-dropdown",
            AudioDeviceType::Source => "source-dropdown",
        }
    }

    fn no_device_message(&self) -> &'static str {
        match self {
            AudioDeviceType::Sink => "No output devices",
            AudioDeviceType::Source => "No input devices",
        }
    }

    fn no_streams_message(&self) -> &'static str {
        match self {
            AudioDeviceType::Sink => "No active playback",
            AudioDeviceType::Source => "No active recording",
        }
    }
}

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_audio_device_panel(
        &mut self,
        device_type: AudioDeviceType,
        devices: SmallVec<[AudioDevice; 8]>,
        streams: SmallVec<[AudioStream; 16]>,
        is_dropdown_open: bool,
        on_toggle: impl Fn(&mut Self, &mut Context<Self>) + 'static,
        on_select: impl Fn(&mut Self, u32, &mut Context<Self>) + 'static,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let default_device = devices.iter().find(|d| d.is_default).cloned();

        let options: Vec<_> = devices
            .iter()
            .map(|d| DropdownOption {
                value: d.id,
                label: d.description.clone(),
            })
            .collect();

        div()
            .bg(theme.bg)
            .rounded_md()
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                Dropdown::new(device_type.dropdown_id(), options)
                    .selected(default_device.map(|d| d.id))
                    .placeholder("No device")
                    .open(is_dropdown_open)
                    .on_toggle(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        on_toggle(this, cx);
                    }))
                    .on_select(cx.listener(move |this, id: &u32, _, cx| {
                        on_select(this, *id, cx);
                    })),
            )
            .when(devices.is_empty(), |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(device_type.no_device_message()),
                )
            })
            .child(div().flex().flex_col().gap_1().mt_3().children({
                if streams.is_empty() {
                    vec![div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(device_type.no_streams_message())
                        .into_any_element()]
                } else {
                    streams
                        .iter()
                        .take(5)
                        .map(|stream| self.render_stream_item(stream, &theme, cx))
                        .collect()
                }
            }))
            .into_any_element()
    }
}
