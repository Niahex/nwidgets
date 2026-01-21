mod audio;
mod details;
mod notifications;
mod quick_actions;

use crate::components::{CircularProgress, Dropdown, DropdownOption, Toggle};
use crate::services::audio::AudioService;
use crate::services::bluetooth::BluetoothService;
use crate::services::control_center::{ControlCenterSection, ControlCenterService};
use crate::services::network::NetworkService;
use crate::services::notifications::{NotificationAdded, NotificationService};
use crate::services::system_monitor::SystemMonitorService;
use crate::theme::ActiveTheme;
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;
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
        let system_monitor = SystemMonitorService::global(cx);
        let hyprland = crate::services::hyprland::HyprlandService::global(cx);

        // Enable system monitoring when control center opens
        system_monitor.read(cx).enable_monitoring();

        let audio_state = audio.read(cx).state();

        cx.subscribe(&control_center, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&hyprland, |this, _, _: &crate::services::hyprland::WorkspaceChanged, cx| {
            this.control_center.update(cx, |cc, cx| {
                if cc.is_visible() {
                    cc.close(cx);
                }
            });
        }).detach();
        cx.subscribe(&hyprland, |this, _, _: &crate::services::hyprland::FullscreenChanged, cx| {
            this.control_center.update(cx, |cc, cx| {
                if cc.is_visible() {
                    cc.close(cx);
                }
            });
        }).detach();
        cx.subscribe(&audio, |this, _, _, cx| {
            let audio_state = this.audio.read(cx).state();
            let now = Instant::now();
            if this.last_volume_update.map(|last| now.duration_since(last) > Duration::from_millis(200)).unwrap_or(true) {
                this.last_volume = audio_state.sink_volume;
            }
            if this.last_mic_update.map(|last| now.duration_since(last) > Duration::from_millis(200)).unwrap_or(true) {
                this.last_mic_volume = audio_state.source_volume;
            }
            cx.notify();
        }).detach();
        cx.subscribe(&bluetooth, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&network, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&system_monitor, |_, _, _, cx| cx.notify()).detach();
        cx.subscribe(&notifications, |_, _, _: &NotificationAdded, cx| cx.notify()).detach();

        let vpn_service = network.read(cx).vpn();
        cx.subscribe(&vpn_service, |_, _, _: &crate::services::network::VpnStateChanged, cx| cx.notify()).detach();

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
            last_volume: audio_state.sink_volume,
            last_mic_volume: audio_state.source_volume,
            last_volume_update: None,
            last_mic_update: None,
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
