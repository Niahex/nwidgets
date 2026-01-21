    fn render_audio_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let audio_state = self.audio.read(cx).state();
        let cc_service = self.control_center.read(cx);
        let expanded = cc_service.expanded_section();

        let vol_expanded = expanded == Some(ControlCenterSection::Volume);
        let mic_expanded = expanded == Some(ControlCenterSection::Mic);

        let theme = cx.global::<crate::theme::Theme>();

        let volume_icon = if audio_state.sink_muted {
            "sink-muted"
        } else {
            "sink-high"
        };
        let mic_icon = if audio_state.source_muted {
            "source-muted"
        } else {
            "source-high"
        };

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                // Volume Row
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(theme.surface)
                    .rounded_md()
                    .p_2()
                    .child(Icon::new(volume_icon).size(px(20.)).color(theme.text))
                    .child(
                        div()
                            .flex_1()
                            .h(px(20.))
                            .flex()
                            .items_center()
                            .on_scroll_wheel(cx.listener(
                                |this, event: &ScrollWheelEvent, window, cx| {
                                    let delta_point = event.delta.pixel_delta(window.line_height());
                                    let delta = if delta_point.y > px(0.0) { 5 } else { -5 };
                                    let current = this.last_volume as i32;
                                    let new_volume = (current + delta).clamp(0, 100) as u8;

                                    if new_volume != this.last_volume {
                                        this.last_volume = new_volume;
                                        cx.notify();

                                        let now = Instant::now();
                                        if this
                                            .last_volume_update
                                            .map(|last| {
                                                now.duration_since(last)
                                                    >= Duration::from_millis(30)
                                            })
                                            .unwrap_or(true)
                                        {
                                            this.last_volume_update = Some(now);
                                            this.audio.update(cx, |audio, cx| {
                                                audio.set_sink_volume(new_volume, cx);
                                            });
                                        }
                                    }
                                },
                            ))
                            .child(
                                div()
                                    .flex_1()
                                    .h(px(4.))
                                    .bg(theme.hover)
                                    .rounded(px(2.))
                                    .child(
                                        div()
                                            .w(relative(self.last_volume as f32 / 100.0))
                                            .h_full()
                                            .bg(theme.accent)
                                            .rounded(px(2.)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text)
                            .child(format!("{}%", self.last_volume)),
                    )
                    .child(
                        div()
                            .id("volume-expand")
                            .child(
                                Icon::new(if vol_expanded {
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
                // Volume Expanded Area
                if vol_expanded {
                    self.render_volume_details(cx)
                } else {
                    div().into_any_element()
                },
            )
            .child(
                // Mic Row
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(theme.surface)
                    .rounded_md()
                    .p_2()
                    .child(Icon::new(mic_icon).size(px(20.)).color(theme.text))
                    .child(
                        div()
                            .flex_1()
                            .h(px(20.))
                            .flex()
                            .items_center()
                            .on_scroll_wheel(cx.listener(
                                |this, event: &ScrollWheelEvent, window, cx| {
                                    let delta_point = event.delta.pixel_delta(window.line_height());
                                    let delta = if delta_point.y > px(0.0) { 5 } else { -5 };
                                    let current = this.last_mic_volume as i32;
                                    let new_volume = (current + delta).clamp(0, 100) as u8;

                                    if new_volume != this.last_mic_volume {
                                        this.last_mic_volume = new_volume;
                                        cx.notify();

                                        let now = Instant::now();
                                        if this
                                            .last_mic_update
                                            .map(|last| {
                                                now.duration_since(last)
                                                    >= Duration::from_millis(30)
                                            })
                                            .unwrap_or(true)
                                        {
                                            this.last_mic_update = Some(now);
                                            this.audio.update(cx, |audio, cx| {
                                                audio.set_source_volume(new_volume, cx);
                                            });
                                        }
                                    }
                                },
                            ))
                            .child(
                                div()
                                    .flex_1()
                                    .h(px(4.))
                                    .bg(theme.hover)
                                    .rounded(px(2.))
                                    .child(
                                        div()
                                            .w(relative(self.last_mic_volume as f32 / 100.0))
                                            .h_full()
                                            .bg(theme.accent_alt)
                                            .rounded(px(2.)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text)
                            .child(format!("{}%", self.last_mic_volume)),
                    )
                    .child(
                        div()
                            .id("mic-expand")
                            .child(
                                Icon::new(if mic_expanded {
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
                // Mic Expanded Area
                if mic_expanded {
                    self.render_mic_details(cx)
                } else {
                    div().into_any_element()
                },
            )
    }

