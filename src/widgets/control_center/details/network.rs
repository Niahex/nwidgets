use crate::components::Toggle;
use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_network_details(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let net_state = self.network.read(cx).state();
        let vpn_service = self.network.read(cx).vpn();
        let vpn_state = vpn_service.read(cx).state();

        div()
            .bg(theme.bg)
            .rounded_md()
            .p_3()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(theme.text).child("Network"))
            .child(div().text_xs().text_color(theme.text_muted).child(net_state.ssid().unwrap_or_else(|| "Not connected".into())))
            .when(!vpn_state.connections.is_empty(), |this| {
                this.child(div().h(px(1.)).bg(theme.hover))
                    .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(theme.text).child("VPN"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .children(vpn_state.connections.iter().take(6).enumerate().map(|(idx, vpn)| {
                                let uuid = vpn.uuid.clone();
                                let vpn_service_clone = vpn_service.clone();
                                let connected = vpn.connected;

                                div()
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
                                            .child(div().text_xs().text_color(theme.text).child(vpn.name.clone()))
                                            .child(div().text_xs().text_color(theme.text_muted).child(vpn.vpn_type.clone())),
                                    )
                                    .child(Toggle::new(("vpn-toggle", idx), connected).on_click(move |_, _, cx| {
                                        let uuid = uuid.clone();
                                        vpn_service_clone.update(cx, |vpn_svc, cx| {
                                            if connected {
                                                vpn_svc.disconnect(uuid, cx);
                                            } else {
                                                vpn_svc.connect(uuid, cx);
                                            }
                                        });
                                    }))
                            })),
                    )
            })
            .into_any_element()
    }
}
