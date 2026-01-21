use crate::ui::components::{Dropdown, DropdownOption};
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
        let default_source = sources.iter().find(|s| s.is_default).cloned();
        let is_open = self.source_dropdown_open;

        let options: Vec<_> = sources
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
                    Dropdown::new("source-dropdown", options)
                        .selected(default_source.map(|s| s.id))
                        .placeholder("No device")
                        .open(is_open)
                        .on_toggle(cx.listener(|this, _: &ClickEvent, _, cx| {
                            this.source_dropdown_open = !this.source_dropdown_open;
                            cx.notify();
                        }))
                        .on_select(cx.listener(|this, id: &u32, _, cx| {
                            this.audio.update(cx, |audio, cx| {
                                audio.set_default_source(*id, cx);
                            });
                            this.source_dropdown_open = false;
                            cx.notify();
                        })),
                )
                .when(sources.is_empty(), |this| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child("No input devices"),
                    )
                })
                .child(div().flex().flex_col().gap_1().mt_3().children({
                    let streams = self.audio.read(cx).source_outputs();
                    if streams.is_empty() {
                        vec![div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child("No active recording")
                            .into_any_element()]
                    } else {
                        streams
                            .iter()
                            .take(5)
                            .map(|stream| {
                                self.render_stream_item(stream, &theme, cx)
                            })
                            .collect()
                    }
                })),
        )
        .into_any_element()
    }
}
