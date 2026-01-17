use crate::components::{Dropdown, DropdownOption};
use crate::services::audio::AudioService;
use crate::services::bluetooth::BluetoothService;
use crate::services::control_center::{ControlCenterSection, ControlCenterService};
use crate::services::network::NetworkService;
use crate::services::notifications::{NotificationAdded, NotificationService};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;
use std::time::{Duration, Instant};

pub struct ControlCenterWidget {
    pub focus_handle: FocusHandle,
    control_center: Entity<ControlCenterService>,
    audio: Entity<AudioService>,
    bluetooth: Entity<BluetoothService>,
    network: Entity<NetworkService>,
    notifications: Entity<NotificationService>,
    sink_dropdown_open: bool,
    source_dropdown_open: bool,
    last_volume: u8,
    last_mic_volume: u8,
    last_volume_update: Option<Instant>,
    last_mic_update: Option<Instant>,
}

fn get_stream_display(
    stream: &crate::services::audio::AudioStream,
) -> (SharedString, &'static str, bool) {
    let title = stream.window_title.as_ref().unwrap_or(&stream.app_name);
    let title_lower = title.to_lowercase();

    let (icon, preserve_colors) = if title_lower.contains("youtube") {
        ("youtube", true)
    } else if title_lower.contains("twitch") {
        ("twitch", true)
    } else if title_lower.contains("discord") {
        ("discord", true)
    } else if title_lower.contains("spotify") {
        ("spotify", true)
    } else if title_lower.contains("firefox") {
        ("firefox", true)
    } else if title_lower.contains("chrome") || title_lower.contains("chromium") {
        ("chrome", true)
    } else if title_lower.contains("vlc") {
        ("vlc", true)
    } else {
        ("application-x-executable", false)
    };

    let display_name: SharedString = if title.len() > 40 {
        format!("{}...", &title[..37]).into()
    } else {
        title.clone()
    };

    (display_name, icon, preserve_colors)
}

impl ControlCenterWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let control_center = ControlCenterService::global(cx);
        let audio = AudioService::global(cx);
        let bluetooth = BluetoothService::global(cx);
        let network = NetworkService::global(cx);
        let notifications = NotificationService::global(cx);

        let audio_state = audio.read(cx).state();

        // Subscriptions
        cx.subscribe(&control_center, |_, _, _, cx| cx.notify())
            .detach();

        cx.subscribe(&audio, |this, _, _, cx| {
            let audio_state = this.audio.read(cx).state();
            let now = Instant::now();

            // Sync UI with audio state if no recent user interaction (prevents jumping)
            if this
                .last_volume_update
                .map(|last| now.duration_since(last) > Duration::from_millis(200))
                .unwrap_or(true)
            {
                this.last_volume = audio_state.sink_volume;
            }
            if this
                .last_mic_update
                .map(|last| now.duration_since(last) > Duration::from_millis(200))
                .unwrap_or(true)
            {
                this.last_mic_volume = audio_state.source_volume;
            }
            cx.notify();
        })
        .detach();

        cx.subscribe(&bluetooth, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&network, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&notifications, |_, _, _: &NotificationAdded, cx| {
            cx.notify()
        })
        .detach();

        Self {
            focus_handle: cx.focus_handle(),
            control_center,
            audio,
            bluetooth,
            network,
            notifications,
            sink_dropdown_open: false,
            source_dropdown_open: false,
            last_volume: audio_state.sink_volume,
            last_mic_volume: audio_state.source_volume,
            last_volume_update: None,
            last_mic_update: None,
        }
    }

    fn render_audio_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let audio_state = self.audio.read(cx).state();
        let cc_service = self.control_center.read(cx);
        let expanded = cc_service.expanded_section();

        let vol_expanded = expanded == Some(ControlCenterSection::Volume);
        let mic_expanded = expanded == Some(ControlCenterSection::Mic);

        let theme = cx.global::<crate::theme::Theme>().clone();

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
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .child("Output Device"),
            )
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
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_3()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_muted)
                            .child("Applications"),
                    )
                    .children({
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
                                                            .w(relative(
                                                                stream_volume as f32 / 100.0,
                                                            ))
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
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .child("Input Device"),
            )
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
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_3()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_muted)
                            .child("Applications"),
                    )
                    .children({
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
                                                            .w(relative(
                                                                stream_volume as f32 / 100.0,
                                                            ))
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

    fn render_connectivity_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let bt_state = self.bluetooth.read(cx).state();
        let net_state = self.network.read(cx).state();
        let cc_service = self.control_center.read(cx);
        let expanded = cc_service.expanded_section();

        let bt_expanded = expanded == Some(ControlCenterSection::Bluetooth);
        let net_expanded = expanded == Some(ControlCenterSection::Network);

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
                        // Bluetooth Button
                        div()
                            .id("bluetooth-toggle")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .bg(if bt_state.powered {
                                theme.accent
                            } else {
                                theme.surface
                            })
                            .rounded_md()
                            .p_4()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Bluetooth, cx);
                                });
                            }))
                            .child(Icon::new("bluetooth").size(px(24.)).color(theme.text))
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
                            .bg(if net_state.connected {
                                theme.accent
                            } else {
                                theme.surface
                            })
                            .rounded_md()
                            .p_4()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle_section(ControlCenterSection::Network, cx);
                                });
                            }))
                            .child(
                                Icon::new(net_state.get_icon_name())
                                    .size(px(24.))
                                    .color(theme.text),
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
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text)
                                .child("Bluetooth Devices"),
                        )
                        .child(div().text_xs().text_color(theme.text_muted).child(format!(
                            "{} device(s) connected",
                            bt_state.connected_devices
                        )))
                        .into_any_element()
                } else if net_expanded {
                    div()
                        .bg(theme.bg)
                        .rounded_md()
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text)
                                .child("Network"),
                        )
                        .child(
                            div().text_xs().text_color(theme.text_muted).child(
                                net_state
                                    .ssid
                                    .clone()
                                    .unwrap_or_else(|| "Not connected".into()),
                            ),
                        )
                        .into_any_element()
                } else {
                    div().into_any_element()
                },
            )
    }

    fn render_notifications_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let notifications = self.notifications.read(cx).get_all();
        let theme = cx.global::<crate::theme::Theme>();
        let notif_service = self.notifications.clone();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(16.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text)
                            .child("Notifications"),
                    )
                    .when(!notifications.is_empty(), |this| {
                        this.child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(theme.surface)
                                .text_xs()
                                .text_color(theme.text_muted)
                                .cursor_pointer()
                                .hover(|style| style.bg(theme.overlay))
                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |_, _, _, cx| {
                                    notif_service.read(cx).clear();
                                    cx.notify();
                                }))
                                .child("Clear"),
                        )
                    }),
            )
            .children(notifications.iter().take(5).map(|n| {
                div()
                    .bg(theme.surface)
                    .rounded_md()
                    .p_2()
                    .mb_1()
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text)
                            .child(n.summary.clone()),
                    )
                    .when(!n.body.is_empty(), |this| {
                        this.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.text_muted)
                                .child(n.body.clone()),
                        )
                    })
            }))
            .when(notifications.is_empty(), |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("No notifications"),
                )
            })
    }
}

impl Render for ControlCenterWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.focus(&self.focus_handle, cx);

        let theme = cx.global::<crate::theme::Theme>().clone();

        div()
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg)
            .rounded(px(18.))
            .overflow_hidden()
            .border_1()
            .border_color(theme.accent_alt.opacity(0.25))
            .shadow_lg()
            .text_color(theme.text)
            .p_4()
            .gap_4()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.key == "escape" {
                    this.control_center.update(cx, |cc, cx| {
                        cc.toggle(cx);
                    });
                }
            }))
            .child(self.render_audio_section(cx))
            .child(self.render_connectivity_section(cx))
            .child(div().h(px(1.)).bg(theme.hover))
            .child(self.render_notifications_section(cx))
    }
}
