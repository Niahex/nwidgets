    fn render_connectivity_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let bt_state = self.bluetooth.read(cx).state();
        let net_state = self.network.read(cx).state();
        let cc_service = self.control_center.read(cx);
        let expanded = cc_service.expanded_section();

        let bt_expanded = expanded == Some(ControlCenterSection::Bluetooth);
        let net_expanded = expanded == Some(ControlCenterSection::Network);
        let monitor_expanded = expanded == Some(ControlCenterSection::Monitor);

        let theme = cx.global::<crate::theme::Theme>();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        // Monitor Button
                        div()
                            .id("monitor-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .font_weight(if monitor_expanded {
                                FontWeight::BOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .when(monitor_expanded, |this| {
                                this.bg(theme.accent.opacity(0.2)).text_color(theme.accent)
                            })
                            .when(!monitor_expanded, |this| {
                                this.text_color(theme.text_muted.opacity(0.5))
                                    .hover(|style| {
                                        style
                                            .bg(theme.hover)
                                            .text_color(theme.text_muted.opacity(0.8))
                                    })
                            })
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Monitor, cx);
                                });
                            }))
                            .child(
                                Icon::new("monitor")
                                    .size(px(20.))
                                    .color(if monitor_expanded {
                                        theme.accent
                                    } else {
                                        theme.text_muted.opacity(0.5)
                                    }),
                            ),
                    )
                    .child(
                        // Bluetooth Button
                        div()
                            .id("bluetooth-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .font_weight(if bt_expanded {
                                FontWeight::BOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .when(bt_expanded, |this| {
                                this.bg(theme.accent.opacity(0.2)).text_color(theme.accent)
                            })
                            .when(!bt_expanded, |this| {
                                this.text_color(theme.text_muted.opacity(0.5))
                                    .hover(|style| {
                                        style
                                            .bg(theme.hover)
                                            .text_color(theme.text_muted.opacity(0.8))
                                    })
                            })
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Bluetooth, cx);
                                });
                            }))
                            .on_mouse_down(
                                gpui::MouseButton::Right,
                                cx.listener(|this, _, _, cx| {
                                    this.bluetooth.update(cx, |bt, cx| {
                                        bt.toggle_power(cx);
                                    });
                                }),
                            )
                            .child(Icon::new("bluetooth-active").size(px(20.)).color(
                                if bt_expanded {
                                    theme.accent
                                } else {
                                    theme.text_muted.opacity(0.5)
                                },
                            ))
                            .when(bt_state.connected_devices > 0, |this| {
                                this.child(format!("{}", bt_state.connected_devices))
                            }),
                    )
                    .child(
                        // Network Button
                        div()
                            .id("network-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .font_weight(if net_expanded {
                                FontWeight::BOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .when(net_expanded, |this| {
                                this.bg(theme.accent.opacity(0.2)).text_color(theme.accent)
                            })
                            .when(!net_expanded, |this| {
                                this.text_color(theme.text_muted.opacity(0.5))
                                    .hover(|style| {
                                        style
                                            .bg(theme.hover)
                                            .text_color(theme.text_muted.opacity(0.8))
                                    })
                            })
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Network, cx);
                                });
                            }))
                            .child(Icon::new(net_state.get_icon_name()).size(px(20.)).color(
                                if net_expanded {
                                    theme.accent
                                } else {
                                    theme.text_muted.opacity(0.5)
                                },
                            )),
                    )
                    .child(
                        // Proxy Button
                        div()
                            .id("proxy-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_muted.opacity(0.5))
                            .hover(|style| {
                                style
                                    .bg(theme.hover)
                                    .text_color(theme.text_muted.opacity(0.8))
                            })
                            .cursor_pointer()
                            .child(
                                Icon::new("proxy")
                                    .size(px(20.))
                                    .color(theme.text_muted.opacity(0.5)),
                            ),
                    )
                    .child(
                        // SSH Button
                        div()
                            .id("ssh-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_muted.opacity(0.5))
                            .hover(|style| {
                                style
                                    .bg(theme.hover)
                                    .text_color(theme.text_muted.opacity(0.8))
                            })
                            .cursor_pointer()
                            .child(
                                Icon::new("ssh")
                                    .size(px(20.))
                                    .color(theme.text_muted.opacity(0.5)),
                            ),
                    )
                    .child(
                        // VM Button
                        div()
                            .id("vm-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_muted.opacity(0.5))
                            .hover(|style| {
                                style
                                    .bg(theme.hover)
                                    .text_color(theme.text_muted.opacity(0.8))
                            })
                            .cursor_pointer()
                            .child(
                                Icon::new("vm")
                                    .size(px(20.))
                                    .color(theme.text_muted.opacity(0.5)),
                            ),
                    ),
            )
            .child(
                // Expanded Area (Shared)
                if bt_expanded {
                    div()
                        .bg(theme.bg)
                        .rounded_md()
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(bt_state.devices.iter().take(8).enumerate().map(|(idx, device)| { // Lazy: max 8 devices
                            let address_for_toggle = device.address.clone();
                            let address_for_pin = device.address.clone();
                            let bluetooth_for_toggle = self.bluetooth.clone();
                            let bluetooth_for_pin = self.bluetooth.clone();
                            div()
                                .id(("bt-device", idx))
                                .flex()
                                .items_center()
                                .justify_between()
                                .p_2()
                                .rounded_md()
                                .bg(theme.surface)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.text)
                                                .child(device.name.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.text_muted)
                                                .child(device.address.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child({
                                            Toggle::new(("bt-toggle", idx), device.connected)
                                                .on_click(move |_, _, cx| {
                                                    let a = address_for_toggle.clone();
                                                    bluetooth_for_toggle.update(cx, |bt, cx| {
                                                        bt.toggle_device(a, cx);
                                                    });
                                                })
                                        })
                                        .child({
                                            div()
                                                .id(("bt-pin", idx))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .p_2()
                                                .rounded_md()
                                                .bg(if device.auto_connect {
                                                    theme.accent.opacity(0.2)
                                                } else {
                                                    theme.hover
                                                })
                                                .cursor_pointer()
                                                .hover(|style| style.bg(theme.accent.opacity(0.3)))
                                                .on_click(move |_, _, cx| {
                                                    let a = address_for_pin.clone();
                                                    bluetooth_for_pin.update(cx, |bt, cx| {
                                                        bt.toggle_auto_connect(a, cx);
                                                    });
                                                })
                                                .child(
                                                    Icon::new(if device.auto_connect {
                                                        "pin"
                                                    } else {
                                                        "unpin"
                                                    })
                                                    .size(px(20.))
                                                    .preserve_colors(!device.auto_connect)
                                                    .color(if device.auto_connect {
                                                        theme.accent
                                                    } else {
                                                        theme.text_muted
                                                    }),
                                                )
                                        }),
                                )
                                .into_any_element()
                        }))
                        .into_any_element()
                } else if net_expanded {
                    self.render_network_details(cx)
                } else if monitor_expanded {
                    self.render_monitor_details(cx)
                } else {
                    div().into_any_element()
                },
            )
    }

