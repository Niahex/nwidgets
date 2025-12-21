use crate::services::audio::AudioService;
use crate::services::bluetooth::BluetoothService;
use crate::services::control_center::{ControlCenterSection, ControlCenterService};
use crate::services::network::NetworkService;
use crate::services::notifications::{NotificationAdded, NotificationService};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;

pub struct ControlCenterWidget {
    control_center: Entity<ControlCenterService>,
    audio: Entity<AudioService>,
    bluetooth: Entity<BluetoothService>,
    network: Entity<NetworkService>,
    notifications: Entity<NotificationService>,
}

impl ControlCenterWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let control_center = ControlCenterService::global(cx);
        let audio = AudioService::global(cx);
        let bluetooth = BluetoothService::global(cx);
        let network = NetworkService::global(cx);
        let notifications = NotificationService::global(cx);

        // Subscriptions
        cx.subscribe(&control_center, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&audio, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&bluetooth, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&network, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&notifications, |_, _, _: &NotificationAdded, cx| cx.notify()).detach();

        Self {
            control_center,
            audio,
            bluetooth,
            network,
            notifications,
        }
    }

    fn render_audio_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let audio_state = self.audio.read(cx).state();
        let cc_service = self.control_center.read(cx);
        let expanded = cc_service.expanded_section();

        let vol_expanded = expanded == Some(ControlCenterSection::Volume);
        let mic_expanded = expanded == Some(ControlCenterSection::Mic);

        // Nord colors
        let bg_color = rgb(0x3b4252); // polar1
        let text_color = rgb(0xeceff4); // snow3
        let active_color = rgb(0x8fbcbb); // frost3

        let volume_icon = if audio_state.sink_muted { "sink-muted" } else { "sink-high" };
        let mic_icon = if audio_state.source_muted { "source-muted" } else { "source-high" };

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
                    .bg(bg_color)
                    .rounded_md()
                    .p_2()
                    .child(Icon::new(volume_icon).size(px(20.)).color(text_color))
                    .child(
                        div() // Slider bar placeholder
                            .flex_1()
                            .h(px(6.))
                            .bg(rgb(0x4c566a))
                            .rounded_full()
                            .child(
                                div()
                                    .w(relative(audio_state.sink_volume as f32 / 100.0))
                                    .h_full()
                                    .bg(active_color)
                                    .rounded_full()
                            )
                    )
                    .child(
                        div()
                            .id("volume-expand")
                            .child(Icon::new(if vol_expanded { "arrow-up" } else { "arrow-down" }).size(px(16.)).color(text_color))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Volume, cx);
                                });
                            }))
                            .cursor_pointer()
                    )
            )
            .child(
                // Volume Expanded Area
                if vol_expanded {
                    div().bg(rgb(0x2e3440)).p_2().child("Volume Details (TODO)").into_any_element()
                } else {
                    div().into_any_element()
                }
            )
            .child(
                // Mic Row
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(bg_color)
                    .rounded_md()
                    .p_2()
                    .child(Icon::new(mic_icon).size(px(20.)).color(text_color))
                    .child(
                        div() // Slider bar placeholder
                            .flex_1()
                            .h(px(6.))
                            .bg(rgb(0x4c566a))
                            .rounded_full()
                            .child(
                                div()
                                    .w(relative(audio_state.source_volume as f32 / 100.0))
                                    .h_full()
                                    .bg(active_color)
                                    .rounded_full()
                            )
                    )
                    .child(
                         div()
                            .id("mic-expand")
                            .child(Icon::new(if mic_expanded { "arrow-up" } else { "arrow-down" }).size(px(16.)).color(text_color))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Mic, cx);
                                });
                            }))
                            .cursor_pointer()
                    )
            )
            .child(
                // Mic Expanded Area
                if mic_expanded {
                    div().bg(rgb(0x2e3440)).p_2().child("Mic Details (TODO)").into_any_element()
                } else {
                    div().into_any_element()
                }
            )
    }

    fn render_connectivity_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let bt_state = self.bluetooth.read(cx).state();
        let net_state = self.network.read(cx).state();
        let cc_service = self.control_center.read(cx);
        let expanded = cc_service.expanded_section();

        let bt_expanded = expanded == Some(ControlCenterSection::Bluetooth);
        let net_expanded = expanded == Some(ControlCenterSection::Network);

        let bg_color = rgb(0x3b4252);
        let active_bg = rgb(0x5e81ac); // polar4
        let text_color = rgb(0xeceff4);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        // Bluetooth Button
                        div()
                            .id("bluetooth-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .bg(if bt_state.powered { active_bg } else { bg_color })
                            .rounded_md()
                            .p_4()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Bluetooth, cx);
                                });
                            }))
                            .child(Icon::new("bluetooth").size(px(24.)).color(text_color))
                            .child(if bt_state.connected_devices > 0 {
                                format!("{}", bt_state.connected_devices)
                            } else {
                                "".to_string()
                            })
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
                            .bg(if net_state.connected { active_bg } else { bg_color })
                            .rounded_md()
                            .p_4()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Network, cx);
                                });
                            }))
                            .child(Icon::new(net_state.get_icon_name()).size(px(24.)).color(text_color))
                    )
            )
            .child(
                // Expanded Area (Shared)
                if bt_expanded {
                    div().bg(rgb(0x2e3440)).p_2().child("Bluetooth Devices (TODO)").into_any_element()
                } else if net_expanded {
                    div().bg(rgb(0x2e3440)).p_2().child(
                        format!("Network: {:?}", net_state.ssid)
                    ).into_any_element()
                } else {
                    div().into_any_element()
                }
            )
    }

    fn render_notifications_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let notifications = self.notifications.read(cx).get_all();
        let text_color = rgb(0xeceff4);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_size(px(16.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_color)
                    .child("Notifications")
            )
            .children(
                notifications.iter().take(5).map(|n| {
                    div()
                        .bg(rgb(0x3b4252))
                        .rounded_md()
                        .p_2()
                        .mb_1()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child(n.summary.clone())
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(rgb(0xd8dee9))
                                .child(n.body.clone())
                        )
                })
            )
    }
}


impl Render for ControlCenterWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x2e3440)) // polar0
            .text_color(rgb(0xeceff4))
            .p_4()
            .gap_4()
            .child(self.render_audio_section(cx))
            .child(self.render_connectivity_section(cx))
            .child(
                div().h(px(1.)).bg(rgb(0x4c566a)) // Separator
            )
            .child(self.render_notifications_section(cx))
    }
}
