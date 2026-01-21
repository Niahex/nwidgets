use crate::components::{Dropdown, DropdownOption};
use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_sink_details(
        &mut self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let sinks = self.audio.read(cx).sinks();
        let default_sink = sinks.iter().find(|s| s.is_default).cloned();
        let is_open = self.sink_dropdown_open;

        let options: Vec<_> = sinks
            .iter()
            .map(|s| DropdownOption {
                value: s.id,
                label: s.description.clone(),
            })
            .collect();

        deferred(
            div()
                .bg(theme.bg)
                .rounded_md()
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    Dropdown::new("sink-dropdown", options)
                        .selected(default_sink.map(|s| s.id))
                        .placeholder("No device")
                        .open(is_open)
                        .on_toggle(cx.listener(|this, _: &ClickEvent, _, cx| {
                            this.sink_dropdown_open = !this.sink_dropdown_open;
                            cx.notify();
                        }))
                        .on_select(cx.listener(|this, id: &u32, _, cx| {
                            this.audio.update(cx, |audio, cx| {
                                audio.set_default_sink(*id, cx);
                            });
                            this.sink_dropdown_open = false;
                            cx.notify();
                        })),
                )
                .when(sinks.is_empty(), |this| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child("No output devices"),
                    )
                })
                .child(div().flex().flex_col().gap_1().mt_3().children({
                    let streams = self.audio.read(cx).sink_inputs();
                    if streams.is_empty() {
                        vec![div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child("No active playback")
                            .into_any_element()]
                    } else {
                        streams
                            .iter()
                            .take(5)
                            .map(|stream| Self::render_stream_item(stream, &theme, theme.accent))
                            .collect()
                    }
                })),
        )
        .into_any_element()
    }
}
