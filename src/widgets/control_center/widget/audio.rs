use crate::services::media::audio::AudioService;
use crate::widgets::control_center::ControlCenterSection;
use crate::theme::{ActiveTheme, Theme};
use crate::assets::Icon;
use crate::components::Slider;
use gpui::*;

impl super::ControlCenterWidget {
    // Helper: render stream item (used by both sink and source)
    pub(in crate::widgets::control_center) fn render_stream_item(
        &mut self,
        stream: &crate::services::media::audio::AudioStream,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let stream_volume = stream.volume;
        let (display_name, icon_name, preserve_colors) = super::control_center_widget::get_stream_display(stream);
        
        let slider = self.get_or_create_stream_slider(stream.id, stream_volume, stream.is_sink_input, cx);

        div()
            .flex()
            .flex_col()
            .gap_1()
            .p_2()
            .bg(theme.surface)
            .rounded_md()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Icon::new(icon_name)
                            .size(px(20.))
                            .preserve_colors(preserve_colors),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(theme.text)
                            .child(display_name),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(format!("{stream_volume}%")),
                    ),
            )
            .child(
                div()
                    .h(px(20.))
                    .flex()
                    .items_center()
                    .child(Slider::new(&slider))
            )
            .into_any_element()
    }

    pub(super) fn render_audio_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let audio_state = self.audio.read(cx).state();
        let cc_service = self.control_center.read(cx);
        let expanded = cc_service.expanded_section();

        let sink_expanded = expanded == Some(ControlCenterSection::Volume);
        let source_expanded = expanded == Some(ControlCenterSection::Mic);

        let theme = cx.theme().clone();

        let sink_icon = if audio_state.sink_muted {
            "sink-muted"
        } else {
            "sink-high"
        };
        let source_icon = if audio_state.source_muted {
            "source-muted"
        } else {
            "source-high"
        };

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                // Sink Row
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(theme.surface)
                    .rounded_md()
                    .p_2()
                    .child(Icon::new(sink_icon).size(px(20.)).color(theme.text))
                    .child(
                        div()
                            .flex_1()
                            .h(px(20.))
                            .flex()
                            .items_center()
                            .child(Slider::new(&self.sink_slider))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text)
                            .child(format!("{}%", self.sink_slider.read(cx).value() as u8)),
                    )
                    .child(
                        div()
                            .id("sink-expand")
                            .child(
                                Icon::new(if sink_expanded {
                                    "arrow-up"
                                } else {
                                    "arrow-down"
                                })
                                .size(px(16.))
                                .color(theme.text),
                            )
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Volume, cx);
                                });
                            }))
                            .cursor_pointer(),
                    ),
            )
            .child(
                // Sink Expanded Area
                if sink_expanded {
                    self.render_sink_details(cx)
                } else {
                    div().into_any_element()
                },
            )
            .child(
                // Source Row
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(theme.surface)
                    .rounded_md()
                    .p_2()
                    .child(Icon::new(source_icon).size(px(20.)).color(theme.text))
                    .child(
                        div()
                            .flex_1()
                            .h(px(20.))
                            .flex()
                            .items_center()
                            .child(Slider::new(&self.source_slider))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text)
                            .child(format!("{}%", self.source_slider.read(cx).value() as u8)),
                    )
                    .child(
                        div()
                            .id("source-expand")
                            .child(
                                Icon::new(if source_expanded {
                                    "arrow-up"
                                } else {
                                    "arrow-down"
                                })
                                .size(px(16.))
                                .color(theme.text),
                            )
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Mic, cx);
                                });
                            }))
                            .cursor_pointer(),
                    ),
            )
            .child(
                // Source Expanded Area
                if source_expanded {
                    self.render_source_details(cx)
                } else {
                    div().into_any_element()
                },
            )
    }
}
