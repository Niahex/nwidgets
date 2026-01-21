use crate::components::Toggle;
use crate::theme::ActiveTheme;
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_bluetooth_details(
        &mut self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let bt_state = self.bluetooth.read(cx).state();
        let theme = cx.theme();

        deferred(
            div()
                .bg(theme.bg)
                .rounded_md()
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                .children(
                    bt_state
                        .devices
                        .iter()
                        .take(8)
                        .enumerate()
                        .map(|(idx, device)| {
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
                                        .child(
                                            Toggle::new(("bt-toggle", idx), device.connected)
                                                .on_click(move |_, _, cx| {
                                                    let a = address_for_toggle.clone();
                                                    bluetooth_for_toggle.update(cx, |bt, cx| {
                                                        bt.toggle_device(a, cx)
                                                    });
                                                }),
                                        )
                                        .child(
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
                                                        bt.toggle_auto_connect(a, cx)
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
                                                ),
                                        ),
                                )
                                .into_any_element()
                        }),
                ),
        )
        .into_any_element()
    }
}
