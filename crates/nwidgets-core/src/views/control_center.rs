use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::corner::{Corner, CornerPosition};
use gpui_component::scroll::ScrollableElement;
use gpui_component::slider::{Slider, SliderState};
use gpui_component::switch::Switch;
use gpui_component::{Icon, Selectable, Sizable};
use nwidgets_service_audio::{AudioService, AudioStateChanged};
use nwidgets_service_bluetooth::{BluetoothService, BluetoothStateChanged};
use nwidgets_service_network::{NetworkService, NetworkStateChanged};
use nwidgets_service_notification::{NotificationAdded, NotificationService, NotificationsCleared};
use nwidgets_service_system_monitor::{SystemMonitorService, SystemStatsChanged};

const CORNER_RADIUS: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCenterSection {
    Monitor,
    Bluetooth,
    Network,
    AudioSink,
    AudioSource,
}

pub struct ControlCenter {
    audio: Entity<AudioService>,
    system_monitor: Entity<SystemMonitorService>,
    bluetooth: Entity<BluetoothService>,
    network: Entity<NetworkService>,
    notifications: Entity<NotificationService>,
    volume_slider: Entity<SliderState>,
    mic_slider: Entity<SliderState>,
    expanded_section: Option<ControlCenterSection>,
}

impl ControlCenter {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);
        let system_monitor = SystemMonitorService::global(cx);
        let bluetooth = BluetoothService::global(cx);
        let network = NetworkService::global(cx);
        let notifications = NotificationService::init(cx);

        let sink_vol = audio.read(cx).state.sink_volume as f32;
        let source_vol = audio.read(cx).state.source_volume as f32;

        let volume_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(1.0)
                .default_value(sink_vol)
        });

        let mic_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(1.0)
                .default_value(source_vol)
        });

        // Subscriptions to update UI when background services emit state changes
        cx.subscribe(&audio, |this, _, _: &AudioStateChanged, cx| {
            let state = this.audio.read(cx).state.clone();
            let sink_vol = state.sink_volume as f32;
            let source_vol = state.source_volume as f32;
            this.volume_slider.update(cx, |slider, _cx| {
                *slider = SliderState::new()
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .default_value(sink_vol);
            });
            this.mic_slider.update(cx, |slider, _cx| {
                *slider = SliderState::new()
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .default_value(source_vol);
            });
            cx.notify();
        })
        .detach();

        cx.subscribe(&volume_slider, |this, _, ev: &gpui_component::slider::SliderEvent, cx| {
            let val = match ev {
                gpui_component::slider::SliderEvent::Change(v) | gpui_component::slider::SliderEvent::Release(v) => {
                    match v {
                        gpui_component::slider::SliderValue::Single(f) => *f,
                        _ => 0.0,
                    }
                }
            };
            this.audio.update(cx, |audio, cx| {
                audio.set_sink_volume(val as u8, cx);
            });
        })
        .detach();

        cx.subscribe(&mic_slider, |this, _, ev: &gpui_component::slider::SliderEvent, cx| {
            let val = match ev {
                gpui_component::slider::SliderEvent::Change(v) | gpui_component::slider::SliderEvent::Release(v) => {
                    match v {
                        gpui_component::slider::SliderValue::Single(f) => *f,
                        _ => 0.0,
                    }
                }
            };
            this.audio.update(cx, |audio, cx| {
                audio.set_source_volume(val as u8, cx);
            });
        })
        .detach();

        cx.subscribe(&system_monitor, |_, _, _: &SystemStatsChanged, cx| cx.notify()).detach();
        cx.subscribe(&bluetooth, |_, _, _: &BluetoothStateChanged, cx| cx.notify()).detach();
        cx.subscribe(&network, |_, _, _: &NetworkStateChanged, cx| cx.notify()).detach();
        cx.subscribe(&notifications, |_, _, _: &NotificationAdded, cx| cx.notify()).detach();
        cx.subscribe(&notifications, |_, _, _: &NotificationsCleared, cx| cx.notify()).detach();

        Self {
            audio,
            system_monitor,
            bluetooth,
            network,
            notifications,
            volume_slider,
            mic_slider,
            expanded_section: None,
        }
    }

    fn toggle_section(&mut self, section: ControlCenterSection, cx: &mut Context<Self>) {
        if self.expanded_section == Some(section) {
            self.expanded_section = None;
        } else {
            self.expanded_section = Some(section);
        }
        cx.notify();
    }

    // ── 1. Audio Section with Dropdown Device Details ──
    fn render_audio_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let card_bg = rgb(0x3b4252);
        let text_main = rgb(0xe5e9f0);
        let text_muted = rgb(0x4c566a);
        let accent = rgb(0x88c0d0);

        let sink_expanded = self.expanded_section == Some(ControlCenterSection::AudioSink);
        let source_expanded = self.expanded_section == Some(ControlCenterSection::AudioSource);

        let audio_state = self.audio.read(cx).state.clone();

        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_3()
            .bg(card_bg)
            .rounded_md()
            // Speaker / Output Volume
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .id("sink-mute-trigger")
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        this.audio.update(cx, |audio, cx| audio.toggle_sink_mute(cx));
                                    }))
                                    .child(Icon::new(if audio_state.sink_muted { "volume_off" } else { "volume_up" }).size(px(20.0)).text_color(if audio_state.sink_muted { rgb(0xbf616a) } else { accent }))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(text_main)
                                            .child(if audio_state.sink_muted { "Output Muted" } else { "Output Volume" }),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(text_muted)
                                            .child(format!("{}%", audio_state.sink_volume)),
                                    )
                                    .child(
                                        Button::new("sink-details-btn")
                                            .ghost()
                                            .with_size(gpui_component::Size::Small)
                                            .icon(Icon::new(if sink_expanded { "keyboard_arrow_up" } else { "keyboard_arrow_down" }).size(px(16.0)))
                                            .on_click(cx.listener(|this, _, _window, cx| {
                                                this.toggle_section(ControlCenterSection::AudioSink, cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .h(px(24.0))
                            .flex()
                            .items_center()
                            .child(Slider::new(&self.volume_slider)),
                    ),
            )
            .when(sink_expanded, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_2()
                        .bg(rgb(0x2e3440))
                        .rounded_md()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(div().text_xs().text_color(text_main).child("Default Speaker (PulseAudio / PipeWire)"))
                                .child(Icon::new("check").size(px(16.0)).text_color(accent)),
                        ),
                )
            })
            // Microphone Input
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .id("source-mute-trigger")
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        this.audio.update(cx, |audio, cx| audio.toggle_source_mute(cx));
                                    }))
                                    .child(Icon::new(if audio_state.source_muted { "mic_off" } else { "mic" }).size(px(20.0)).text_color(if audio_state.source_muted { rgb(0xbf616a) } else { accent }))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(text_main)
                                            .child(if audio_state.source_muted { "Microphone Muted" } else { "Microphone" }),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(text_muted)
                                            .child(format!("{}%", audio_state.source_volume)),
                                    )
                                    .child(
                                        Button::new("source-details-btn")
                                            .ghost()
                                            .with_size(gpui_component::Size::Small)
                                            .icon(Icon::new(if source_expanded { "keyboard_arrow_up" } else { "keyboard_arrow_down" }).size(px(16.0)))
                                            .on_click(cx.listener(|this, _, _window, cx| {
                                                this.toggle_section(ControlCenterSection::AudioSource, cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .h(px(24.0))
                            .flex()
                            .items_center()
                            .child(Slider::new(&self.mic_slider)),
                    ),
            )
            .when(source_expanded, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_2()
                        .bg(rgb(0x2e3440))
                        .rounded_md()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(div().text_xs().text_color(text_main).child("Default Internal Microphone"))
                                .child(Icon::new("check").size(px(16.0)).text_color(accent)),
                        ),
                )
            })
    }

    // ── 2. Quick Actions & Connectivity Section ──
    fn render_quick_actions(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let monitor_expanded = self.expanded_section == Some(ControlCenterSection::Monitor);
        let bt_expanded = self.expanded_section == Some(ControlCenterSection::Bluetooth);
        let net_expanded = self.expanded_section == Some(ControlCenterSection::Network);

        let stats = self.system_monitor.read(cx).stats.clone();
        let bt_state = self.bluetooth.read(cx).state.clone();
        let net_state = self.network.read(cx).state.clone();

        let bt_active = bt_state.powered;
        let wifi_active = net_state.wifi_enabled;

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .gap_2()
                    // System Monitor Toggle
                    .child(
                        div().flex_1().child(
                            Button::new("monitor-toggle")
                                .secondary()
                                .with_size(gpui_component::Size::Medium)
                                .icon(Icon::new("monitor").size(px(20.0)))
                                .selected(monitor_expanded)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.toggle_section(ControlCenterSection::Monitor, cx);
                                })),
                        ),
                    )
                    // Bluetooth Toggle & Details
                    .child(
                        div().flex_1().child(
                            Button::new("bt-toggle")
                                .secondary()
                                .with_size(gpui_component::Size::Medium)
                                .icon(Icon::new(if bt_active { "bluetooth" } else { "bluetooth_disabled" }).size(px(20.0)))
                                .selected(bt_expanded)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.toggle_section(ControlCenterSection::Bluetooth, cx);
                                })),
                        ),
                    )
                    // Network / Wi-Fi Toggle & Details
                    .child(
                        div().flex_1().child(
                            Button::new("network-toggle")
                                .secondary()
                                .with_size(gpui_component::Size::Medium)
                                .icon(Icon::new(if wifi_active { "wifi" } else { "wifi_off" }).size(px(20.0)))
                                .selected(net_expanded)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.toggle_section(ControlCenterSection::Network, cx);
                                })),
                        ),
                    )
                    // Proxy Toggle
                    .child(
                        div().flex_1().child(
                            Button::new("proxy-toggle")
                                .secondary()
                                .with_size(gpui_component::Size::Medium)
                                .icon(Icon::new("vpn_key").size(px(20.0)))
                                .on_click(|_, _, _| {}),
                        ),
                    )
                    // SSH Toggle
                    .child(
                        div().flex_1().child(
                            Button::new("ssh-toggle")
                                .secondary()
                                .with_size(gpui_component::Size::Medium)
                                .icon(Icon::new("terminal").size(px(20.0)))
                                .on_click(|_, _, _| {}),
                        ),
                    ),
            )
            // ── Expandable Details Panels ──
            .when(monitor_expanded, |div_elem| {
                div_elem.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_3()
                        .bg(rgb(0x3b4252))
                        .rounded_md()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x88c0d0))
                                .child("System Monitor Stats"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_around()
                                .p_2()
                                .child(
                                    gpui_component::progress::CircularGauge::new("cpu-gauge")
                                        .primary_value(stats.cpu as f32)
                                        .primary_label("CPU")
                                        .primary_unit("%")
                                        .primary_color(rgb(0x88c0d0))
                                        .secondary_value(stats.cpu_temp.unwrap_or(45) as f32)
                                        .secondary_label("Temp")
                                        .secondary_unit("°C")
                                        .secondary_color(rgb(0xebcb8b))
                                        .with_size(gpui_component::Size::Medium),
                                )
                                .child(
                                    gpui_component::progress::CircularGauge::new("gpu-gauge")
                                        .primary_value(stats.gpu as f32)
                                        .primary_label("GPU")
                                        .primary_unit("%")
                                        .primary_color(rgb(0x88c0d0))
                                        .secondary_value(stats.gpu_temp.unwrap_or(50) as f32)
                                        .secondary_label("Temp")
                                        .secondary_unit("°C")
                                        .secondary_color(rgb(0xebcb8b))
                                        .with_size(gpui_component::Size::Medium),
                                )
                                .child(
                                    gpui_component::progress::CircularGauge::new("ram-gauge")
                                        .primary_value(stats.ram as f32)
                                        .primary_label("RAM")
                                        .primary_unit("%")
                                        .primary_color(rgb(0x88c0d0))
                                        .secondary_value(65.0)
                                        .secondary_label("Disk")
                                        .secondary_unit("%")
                                        .secondary_color(rgb(0xb48ead))
                                        .with_size(gpui_component::Size::Medium),
                                ),
                        ),
                )
            })
            .when(bt_expanded, |div_elem| {
                let devices = bt_state.devices.clone();
                div_elem.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_3()
                        .bg(rgb(0x3b4252))
                        .rounded_md()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0x88c0d0)).child("Bluetooth Devices"))
                                .child(
                                    Switch::new("bt-switch")
                                        .checked(bt_active)
                                        .on_click(cx.listener(|this, _, _window, cx| {
                                            this.bluetooth.update(cx, |bt, cx| bt.toggle_power(cx));
                                        })),
                                ),
                        )
                        .children(devices.into_iter().map(|dev| {
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .p_2()
                                .bg(rgb(0x2e3440))
                                .rounded_md()
                                .child(div().text_xs().text_color(rgb(0xe5e9f0)).child(dev.name))
                                .child(div().text_xs().text_color(rgb(0xa3be8c)).child(if dev.connected { "Connected" } else { "Paired" }))
                        })),
                )
            })
            .when(net_expanded, |div_elem| {
                let networks = net_state.networks.clone();
                div_elem.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_3()
                        .bg(rgb(0x3b4252))
                        .rounded_md()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0x88c0d0)).child("Wi-Fi Networks"))
                                .child(
                                    Switch::new("wifi-switch")
                                        .checked(wifi_active)
                                        .on_click(cx.listener(|this, _, _window, cx| {
                                            this.network.update(cx, |net, cx| net.toggle_wifi(cx));
                                        })),
                                ),
                        )
                        .children(networks.into_iter().map(|net| {
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .p_2()
                                .bg(rgb(0x2e3440))
                                .rounded_md()
                                .child(div().text_xs().text_color(rgb(0xe5e9f0)).child(net.ssid))
                                .when(net.active, |this| {
                                    this.child(Icon::new("check").size(px(16.0)).text_color(rgb(0x88c0d0)))
                                })
                        })),
                )
            })
    }

    // ── 3. Notifications Section (matching notifications.rs) ──
    fn render_notifications_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let card_bg = rgb(0x3b4252);
        let text_main = rgb(0xe5e9f0);
        let text_muted = rgb(0xd8dee9);
        let accent = rgb(0x88c0d0);

        let notifs = self.notifications.read(cx).history.clone();
        let notif_count = notifs.len();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .bg(card_bg)
            .rounded_md()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(text_main)
                                    .child("Notifications"),
                            )
                            .when(notif_count > 0, |d| {
                                d.child(
                                    div()
                                        .px_1_5()
                                        .py_0p5()
                                        .bg(accent)
                                        .rounded_full()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x2e3440))
                                        .child(format!("{}", notif_count)),
                                )
                            }),
                    )
                    .when(notif_count > 0, |d| {
                        d.child(
                            Button::new("clear-all-notifs")
                                .ghost()
                                .with_size(gpui_component::Size::Small)
                                .label("Clear All")
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.notifications.update(cx, |srv, cx| srv.clear(cx));
                                })),
                        )
                    }),
            )
            .child(if notifs.is_empty() {
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .text_color(text_muted)
                    .child("No notifications")
                    .into_any_element()
            } else {
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .overflow_y_scrollbar()
                    .children(notifs.into_iter().map(|notif| {
                        let notif_id = notif.id;
                        div()
                            .id(SharedString::from(format!("cc-notif-{}", notif_id)))
                            .flex()
                            .flex_col()
                            .gap_1()
                            .p_2()
                            .bg(rgb(0x2e3440))
                            .rounded_md()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(Icon::new("notifications").size(px(14.0)).text_color(accent))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(text_main)
                                                    .child(notif.app_name.clone()),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(text_main)
                                    .child(notif.summary.clone()),
                            )
                            .when(!notif.body.as_ref().is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(text_muted)
                                        .child(notif.body.clone()),
                                )
                            })
                    }))
                    .into_any_element()
            })
    }
}

impl Render for ControlCenter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x2e3440);
        let hover_line = rgb(0x3b4252);

        div()
            .size_full()
            .flex()
            .flex_row()
            // ── Left concave corners ──
            .child(
                div()
                    .h_full()
                    .w(px(CORNER_RADIUS))
                    .flex()
                    .flex_col()
                    .child(
                        Corner::new(CornerPosition::TopRight, px(CORNER_RADIUS)).color(bg),
                    )
                    .child(div().flex_1())
                    .child(
                        Corner::new(CornerPosition::BottomRight, px(CORNER_RADIUS)).color(bg),
                    ),
            )
            // ── Main Control Center Container ──
            .child(
                div()
                    .w_full()
                    .h_full()
                    .bg(bg)
                    .flex()
                    .flex_col()
                    .p_4()
                    .gap_4()
                    // 1. Audio Section (Sink/Source Sliders & Device Dropdowns)
                    .child(self.render_audio_section(cx))
                    .child(div().h(px(1.0)).bg(hover_line))
                    // 2. Connectivity & System Quick Toggles with expandable detail panels
                    .child(self.render_quick_actions(cx))
                    .child(div().h(px(1.0)).bg(hover_line))
                    // 3. Notifications Section
                    .child(self.render_notifications_section(cx)),
            )
    }
}
