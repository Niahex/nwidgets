    fn render_volume_details(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.global::<crate::theme::Theme>().clone();
        let sinks = self.audio.read(cx).sinks();
        let default_sink = sinks.iter().find(|s| s.is_default).cloned();
        let is_open = self.sink_dropdown_open;
        let audio = self.audio.clone();
        let sinks_empty = sinks.is_empty();

        let options: Vec<_> = sinks
            .iter()
            .map(|s| DropdownOption {
                value: s.id,
                label: s.description.clone(),
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
                Dropdown::new("sink-dropdown", options)
                    .selected(default_sink.map(|s| s.id))
                    .placeholder("No device")
                    .open(is_open)
                    .on_toggle(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.sink_dropdown_open = !this.sink_dropdown_open;
                        cx.notify();
                    }))
                    .on_select(cx.listener(move |this, id: &u32, _, cx| {
                        audio.update(cx, |audio, cx| {
                            audio.set_default_sink(*id, cx);
                        });
                        this.sink_dropdown_open = false;
                        cx.notify();
                    })),
            )
            .when(sinks_empty, |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("No output devices"),
                )
            })
            .child(
                // Streams section
                div().flex().flex_col().gap_1().mt_3().children({
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
                            .map(|stream| {
                                let stream_volume = stream.volume;
                                let (display_name, icon_name, preserve_colors) =
                                    get_stream_display(stream);

                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .p_2()
                                    .bg(theme.surface)
                                    .rounded_md()
                                    .child(
                                        // First line: icon + app name + volume %
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
                                        // Second line: volume bar (visual only)
                                        div().h(px(20.)).flex().items_center().child(
                                            div()
                                                .flex_1()
                                                .h(px(4.))
                                                .bg(theme.hover)
                                                .rounded(px(2.))
                                                .child(
                                                    div()
                                                        .w(relative(stream_volume as f32 / 100.0))
                                                        .h_full()
                                                        .bg(theme.accent)
                                                        .rounded(px(2.)),
                                                ),
                                        ),
                                    )
                                    .into_any_element()
                            })
                            .collect()
                    }
                }),
            )
            .into_any_element()
    }

    fn render_mic_details(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.global::<crate::theme::Theme>().clone();
        let sources = self.audio.read(cx).sources();
        let default_source = sources.iter().find(|s| s.is_default).cloned();
        let is_open = self.source_dropdown_open;
        let audio = self.audio.clone();
        let sources_empty = sources.is_empty();

        let options: Vec<_> = sources
            .iter()
            .map(|s| DropdownOption {
                value: s.id,
                label: s.description.clone(),
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
                Dropdown::new("source-dropdown", options)
                    .selected(default_source.map(|s| s.id))
                    .placeholder("No device")
                    .open(is_open)
                    .on_toggle(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.source_dropdown_open = !this.source_dropdown_open;
                        cx.notify();
                    }))
                    .on_select(cx.listener(move |this, id: &u32, _, cx| {
                        audio.update(cx, |audio, cx| {
                            audio.set_default_source(*id, cx);
                        });
                        this.source_dropdown_open = false;
                        cx.notify();
                    })),
            )
            .when(sources_empty, |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("No input devices"),
                )
            })
            .child(
                // Streams section
                div().flex().flex_col().gap_1().mt_3().children({
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
                            .map(|stream| {
                                let stream_volume = stream.volume;
                                let (display_name, icon_name, preserve_colors) =
                                    get_stream_display(stream);

                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .p_2()
                                    .bg(theme.surface)
                                    .rounded_md()
                                    .child(
                                        // First line: icon + app name + volume %
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
                                        // Second line: volume bar (visual only)
                                        div().h(px(20.)).flex().items_center().child(
                                            div()
                                                .flex_1()
                                                .h(px(4.))
                                                .bg(theme.hover)
                                                .rounded(px(2.))
                                                .child(
                                                    div()
                                                        .w(relative(stream_volume as f32 / 100.0))
                                                        .h_full()
                                                        .bg(theme.accent_alt)
                                                        .rounded(px(2.)),
                                                ),
                                        ),
                                    )
                                    .into_any_element()
                            })
                            .collect()
                    }
                }),
            )
            .into_any_element()
    }

