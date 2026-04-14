use crate::components::Button;
use crate::widgets::control_center::ControlCenterSection;
use gpui::prelude::*;
use gpui::*;

impl super::ControlCenterWidget {
    pub(super) fn render_connectivity_section(
        &mut self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let bt_state = self.bluetooth.read(cx).state();
        let net_state = self.network.read(cx).state();
        let cc_service = self.control_center.read(cx);
        let expanded = cc_service.expanded_section();

        let bt_expanded = expanded == Some(ControlCenterSection::Bluetooth);
        let net_expanded = expanded == Some(ControlCenterSection::Network);
        let monitor_expanded = expanded == Some(ControlCenterSection::Monitor);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div().flex_1().child(
                            Button::new("monitor-toggle")
                                .icon("monitor")
                                .icon_size(px(20.))
                                .accent()
                                .selected(monitor_expanded)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.control_center.update(cx, |cc, cx| {
                                        cc.toggle_section(ControlCenterSection::Monitor, cx)
                                    });
                                })),
                        ),
                    )
                    .child(div().flex_1().child({
                        let mut button = Button::new("bluetooth-toggle")
                            .icon("bluetooth-active")
                            .icon_size(px(20.))
                            .accent()
                            .selected(bt_expanded)
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Bluetooth, cx)
                                });
                            }))
                            .on_right_click(cx.listener(|this, _, _, cx| {
                                this.bluetooth.update(cx, |bt, cx| bt.toggle_power(cx));
                            }));

                        if bt_state.connected_devices > 0 {
                            button = button.child(format!("{}", bt_state.connected_devices));
                        }

                        button
                    }))
                    .child(
                        div().flex_1().child(
                            Button::new("network-toggle")
                                .icon(net_state.get_icon_name())
                                .icon_size(px(20.))
                                .accent()
                                .selected(net_expanded)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.control_center.update(cx, |cc, cx| {
                                        cc.toggle_section(ControlCenterSection::Network, cx)
                                    });
                                })),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            Button::new("proxy-toggle")
                                .icon("proxy")
                                .icon_size(px(20.))
                                .accent()
                                .on_click(|_, _, _| {}),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            Button::new("ssh-toggle")
                                .icon("ssh")
                                .icon_size(px(20.))
                                .accent()
                                .on_click(|_, _, _| {}),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            Button::new("vm-toggle")
                                .icon("vm")
                                .icon_size(px(20.))
                                .accent()
                                .on_click(|_, _, _| {}),
                        ),
                    ),
            )
            .child(if bt_expanded {
                self.render_bluetooth_details(cx)
            } else if net_expanded {
                self.render_network_details(cx)
            } else if monitor_expanded {
                self.render_monitor_details(cx)
            } else {
                div().into_any_element()
            })
    }
}
