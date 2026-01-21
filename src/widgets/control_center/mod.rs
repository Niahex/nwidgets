mod audio;
mod details;
mod notifications;
mod quick_actions;

use crate::ui::components::{CircularProgress, Dropdown, DropdownOption, SliderState, Toggle};
use crate::services::media::audio::AudioService;
use crate::services::hardware::bluetooth::BluetoothService;
use crate::services::ui::control_center::{ControlCenterSection, ControlCenterService};
use crate::services::network::NetworkService;
use crate::services::ui::notifications::{NotificationAdded, NotificationService};
use crate::services::hardware::system_monitor::SystemMonitorService;
use crate::theme::ActiveTheme;
use crate::assets::Icon;
use gpui::prelude::*;
use gpui::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

actions!(control_center, [CloseControlCenter]);

pub struct ControlCenterWidget {
    pub focus_handle: FocusHandle,
    control_center: Entity<ControlCenterService>,
    audio: Entity<AudioService>,
    bluetooth: Entity<BluetoothService>,
    network: Entity<NetworkService>,
    notifications: Entity<NotificationService>,
    system_monitor: Entity<SystemMonitorService>,
    sink_dropdown_open: bool,
    source_dropdown_open: bool,
    sink_slider: Entity<SliderState>,
    source_slider: Entity<SliderState>,
    stream_sliders: HashMap<u32, Entity<SliderState>>,
}

fn get_stream_display(
    stream: &crate::services::media::audio::AudioStream,
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
        ("none", false)
    };

    let display_name: SharedString = if title.len() > 40 {
        format!("{}...", &title[..37]).into()
    } else {
        title.clone()
    };

    (display_name, icon, preserve_colors)
}

impl ControlCenterWidget {
    fn get_or_create_stream_slider(
        &mut self,
        stream_id: u32,
        initial_volume: u8,
        is_sink_input: bool,
        cx: &mut Context<Self>,
    ) -> Entity<SliderState> {
        self.stream_sliders
            .entry(stream_id)
            .or_insert_with(|| {
                let slider = cx.new(|_| {
                    SliderState::new()
                        .min(0.0)
                        .max(100.0)
                        .step(5.0)
                        .default_value(initial_volume as f32)
                });
                
                // Subscribe to slider events
                cx.subscribe(&slider, move |this, _, event: &crate::ui::components::SliderEvent, cx| {
                    if let crate::ui::components::SliderEvent::Change(value) = event {
                        this.audio.update(cx, |audio, cx| {
                            if is_sink_input {
                                audio.set_sink_input_volume(stream_id, *value as u8, cx);
                            } else {
                                audio.set_source_output_volume(stream_id, *value as u8, cx);
                            }
                        });
                    }
                })
                .detach();
                
                slider
            })
            .clone()
    }

    pub fn new(cx: &mut Context<Self>) -> Self {
        let control_center = ControlCenterService::global(cx);
        let audio = AudioService::global(cx);
        let bluetooth = BluetoothService::global(cx);
        let network = NetworkService::global(cx);
        let notifications = NotificationService::global(cx);
        let system_monitor = SystemMonitorService::global(cx);
        let hyprland = crate::services::system::hyprland::HyprlandService::global(cx);

        // Enable system monitoring when control center opens
        system_monitor.read(cx).enable_monitoring();

        let audio_state = audio.read(cx).state();

        // Create sliders
        let sink_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(5.0)
                .default_value(audio_state.sink_volume as f32)
        });

        let source_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(5.0)
                .default_value(audio_state.source_volume as f32)
        });

        cx.subscribe(&control_center, |_, _, _, cx| cx.notify())
            .detach();
        cx.subscribe(
            &hyprland,
            |this, _, _: &crate::services::system::hyprland::WorkspaceChanged, cx| {
                this.control_center.update(cx, |cc, cx| {
                    if cc.is_visible() {
                        cc.close(cx);
                    }
                });
            },
        )
        .detach();
        cx.subscribe(
            &hyprland,
            |this, _, _: &crate::services::system::hyprland::FullscreenChanged, cx| {
                this.control_center.update(cx, |cc, cx| {
                    if cc.is_visible() {
                        cc.close(cx);
                    }
                });
            },
        )
        .detach();
        cx.subscribe(&audio, |this, _, _, cx| {
            let audio_state = this.audio.read(cx).state();
            // Update sliders when audio state changes (without emitting events)
            this.sink_slider.update(cx, |slider, cx| {
                slider.update_value(audio_state.sink_volume as f32, cx);
            });
            this.source_slider.update(cx, |slider, cx| {
                slider.update_value(audio_state.source_volume as f32, cx);
            });
            cx.notify();
        })
        .detach();
        cx.subscribe(&bluetooth, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&network, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&system_monitor, |_, _, _, cx| cx.notify())
            .detach();
        cx.subscribe(&notifications, |_, _, _: &NotificationAdded, cx| {
            cx.notify()
        })
        .detach();

        // Subscribe to slider events
        cx.subscribe(&sink_slider, |this, _, event: &crate::ui::components::SliderEvent, cx| {
            if let crate::ui::components::SliderEvent::Change(value) = event {
                this.audio.update(cx, |audio, cx| {
                    audio.set_sink_volume(*value as u8, cx);
                });
            }
        })
        .detach();

        cx.subscribe(&source_slider, |this, _, event: &crate::ui::components::SliderEvent, cx| {
            if let crate::ui::components::SliderEvent::Change(value) = event {
                this.audio.update(cx, |audio, cx| {
                    audio.set_source_volume(*value as u8, cx);
                });
            }
        })
        .detach();

        let vpn_service = network.read(cx).vpn();
        cx.subscribe(
            &vpn_service,
            |_, _, _: &crate::services::network::VpnStateChanged, cx| cx.notify(),
        )
        .detach();

        Self {
            focus_handle: cx.focus_handle(),
            control_center,
            audio,
            bluetooth,
            network,
            notifications,
            system_monitor,
            sink_dropdown_open: false,
            source_dropdown_open: false,
            sink_slider,
            source_slider,
            stream_sliders: HashMap::new(),
        }
    }
}

impl Render for ControlCenterWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.focus(&self.focus_handle, cx);

        let theme = cx.theme().clone();

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
            .on_action(|_: &CloseControlCenter, window, _cx| {
                window.remove_window();
            })
            .child(self.render_audio_section(cx))
            .child(div().h(px(1.)).bg(theme.hover))
            .child(self.render_connectivity_section(cx))
            .child(div().h(px(1.)).bg(theme.hover))
            .child(self.render_notifications_section(cx))
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.system_monitor.read(cx).disable_monitoring();
                this.control_center.update(cx, |cc, cx| {
                    cc.close(cx);
                });
                window.remove_window();
            }))
            .with_animation(
                "control-center-fade-in",
                Animation::new(Duration::from_millis(150)),
                |this, delta| this.opacity(delta),
            )
    }
}

impl Drop for ControlCenterWidget {
    fn drop(&mut self) {
        log::debug!("Control center widget dropped, disabling monitoring");
    }
}
