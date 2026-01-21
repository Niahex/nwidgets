use crate::services::control_center::ControlCenterSection;
use crate::theme::ActiveTheme;
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;

impl super::ControlCenterWidget {
    pub(super) fn render_connectivity_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let bt_state = self.bluetooth.read(cx).state();
        let net_state = self.network.read(cx).state();
        let cc_service = self.control_center.read(cx);
        let expanded = cc_service.expanded_section();

        let bt_expanded = expanded == Some(ControlCenterSection::Bluetooth);
        let net_expanded = expanded == Some(ControlCenterSection::Network);
        let monitor_expanded = expanded == Some(ControlCenterSection::Monitor);

        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
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
                            .font_weight(if monitor_expanded { FontWeight::BOLD } else { FontWeight::MEDIUM })
                            .when(monitor_expanded, |this| this.bg(theme.accent.opacity(0.2)).text_color(theme.accent))
                            .when(!monitor_expanded, |this| {
                                this.text_color(theme.text_muted.opacity(0.5))
                                    .hover(|style| style.bg(theme.hover).text_color(theme.text_muted.opacity(0.8)))
                            })
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| cc.toggle_section(ControlCenterSection::Monitor, cx));
                            }))
                            .child(Icon::new("monitor").size(px(20.)).color(if monitor_expanded { theme.accent } else { theme.text_muted.opacity(0.5) })),
                    )
                    .child(
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
                            .font_weight(if bt_expanded { FontWeight::BOLD } else { FontWeight::MEDIUM })
                            .when(bt_expanded, |this| this.bg(theme.accent.opacity(0.2)).text_color(theme.accent))
                            .when(!bt_expanded, |this| {
                                this.text_color(theme.text_muted.opacity(0.5))
                                    .hover(|style| style.bg(theme.hover).text_color(theme.text_muted.opacity(0.8)))
                            })
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| cc.toggle_section(ControlCenterSection::Bluetooth, cx));
                            }))
                            .on_mouse_down(gpui::MouseButton::Right, cx.listener(|this, _, _, cx| {
                                this.bluetooth.update(cx, |bt, cx| bt.toggle_power(cx));
                            }))
                            .child(Icon::new("bluetooth-active").size(px(20.)).color(if bt_expanded { theme.accent } else { theme.text_muted.opacity(0.5) }))
                            .when(bt_state.connected_devices > 0, |this| this.child(format!("{}", bt_state.connected_devices))),
                    )
                    .child(
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
                            .font_weight(if net_expanded { FontWeight::BOLD } else { FontWeight::MEDIUM })
                            .when(net_expanded, |this| this.bg(theme.accent.opacity(0.2)).text_color(theme.accent))
                            .when(!net_expanded, |this| {
                                this.text_color(theme.text_muted.opacity(0.5))
                                    .hover(|style| style.bg(theme.hover).text_color(theme.text_muted.opacity(0.8)))
                            })
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| cc.toggle_section(ControlCenterSection::Network, cx));
                            }))
                            .child(Icon::new(net_state.get_icon_name()).size(px(20.)).color(if net_expanded { theme.accent } else { theme.text_muted.opacity(0.5) })),
                    )
                    .child(
                        div()
                            .id("proxy-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .text_color(theme.text_muted.opacity(0.5))
                            .hover(|style| style.bg(theme.hover).text_color(theme.text_muted.opacity(0.8)))
                            .cursor_pointer()
                            .child(Icon::new("proxy").size(px(20.)).color(theme.text_muted.opacity(0.5))),
                    )
                    .child(
                        div()
                            .id("ssh-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .text_color(theme.text_muted.opacity(0.5))
                            .hover(|style| style.bg(theme.hover).text_color(theme.text_muted.opacity(0.8)))
                            .cursor_pointer()
                            .child(Icon::new("ssh").size(px(20.)).color(theme.text_muted.opacity(0.5))),
                    )
                    .child(
                        div()
                            .id("vm-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .text_color(theme.text_muted.opacity(0.5))
                            .hover(|style| style.bg(theme.hover).text_color(theme.text_muted.opacity(0.8)))
                            .cursor_pointer()
                            .child(Icon::new("vm").size(px(20.)).color(theme.text_muted.opacity(0.5))),
                    ),
            )
            .child(
                if bt_expanded {
                    self.render_bluetooth_details(cx)
                } else if net_expanded {
                    self.render_network_details(cx)
                } else if monitor_expanded {
                    self.render_monitor_details(cx)
                } else {
                    div().into_any_element()
                },
            )
    }
}
