mod services;
mod utils;
mod widgets;

mod style {
    include!(concat!(env!("OUT_DIR"), "/generated_style.rs"));
}

use crate::services::bluetooth::BluetoothService;
use crate::services::clipboard::ClipboardService;
use crate::services::hyprland::HyprlandService;
use crate::services::lock_state::{CapsLockService, NumLockService};
use crate::services::mpris::MprisService;
use crate::services::network::NetworkService;
use crate::services::osd::OsdEventService;
use crate::services::pipewire::PipeWireService;
use crate::services::stt::SttService;
use crate::services::systray::SystemTrayService;
use crate::widgets::control_center::create_control_center_window;
use crate::widgets::notifications::create_notifications_window;
use crate::widgets::osd::create_osd_window;
use crate::widgets::panel::create_panel_window;
use crate::widgets::power_menu::create_power_menu_window;
use gtk4::{self as gtk, prelude::*, Application};

const APP_ID: &str = "github.niahex.nwidgets";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        style::load_css();

        utils::icons::setup_icon_theme();

        let _power_menu_window = create_power_menu_window(app);

        let _control_center_window = create_control_center_window(app);

        crate::services::NotificationService::subscribe_notifications(|notification| {
            println!(
                "[MAIN] Received notification: {} - {}",
                notification.summary, notification.body
            );
        });

        println!(
            "[MAIN] Notification history size: {}",
            crate::services::NotificationService::get_history().len()
        );

        // Initialize STT service
        let stt_service = std::sync::Arc::new(SttService::new());
        let _ = stt_service.initialize();

        // Create STT toggle action
        let stt_action = gtk::gio::SimpleAction::new("toggle-stt", None);
        let stt_service_clone = stt_service.clone();
        stt_action.connect_activate(move |_, _| {
            if let Err(e) = stt_service_clone.toggle() {
                eprintln!("[STT] Toggle error: {e}");
            }
        });
        app.add_action(&stt_action);

        let (
            panel_window,
            active_window_module,
            workspaces_module,
            mpris_module,
            bluetooth_module,
            network_module,
            systray_module,
            volume_module,
            mic_module,
            _pomodoro_module,
        ) = create_panel_window(app);
        panel_window.present();

        let osd_window = create_osd_window(app);
        osd_window.present();

        let _notifications_window = create_notifications_window(app);

        let active_window_module_clone = active_window_module.clone();
        HyprlandService::subscribe_active_window(move |active_window| {
            active_window_module_clone.update_hyprland_window(active_window.clone());
        });

        let workspaces_module_clone = workspaces_module.clone();
        HyprlandService::subscribe_workspace(move |workspaces, active_workspace| {
            workspaces_module_clone.update(workspaces, active_workspace);
        });

        let mpris_module_clone = mpris_module.clone();
        MprisService::subscribe(move |state| {
            mpris_module_clone.update(state);
        });

        let bluetooth_module_clone = bluetooth_module.clone();
        BluetoothService::subscribe_bluetooth(move |state| {
            bluetooth_module_clone.update(state);
        });

        let network_module_clone = network_module.clone();
        NetworkService::subscribe_network(move |state| {
            network_module_clone.update(state);
        });

        let systray_module_clone = systray_module.clone();
        SystemTrayService::subscribe_systray(move |items| {
            systray_module_clone.update(items);
        });

        let volume_module_clone = volume_module.clone();
        let mic_module_clone = mic_module.clone();
        let last_volume = std::cell::Cell::new(0u8);
        let last_muted = std::cell::Cell::new(false);
        let last_mic_volume = std::cell::Cell::new(0u8);
        let last_mic_muted = std::cell::Cell::new(false);
        PipeWireService::subscribe_audio(move |state| {
            volume_module_clone.update(&state);
            mic_module_clone.update(&state);

            if state.volume != last_volume.get() || state.muted != last_muted.get() {
                OsdEventService::send_event(crate::services::osd::OsdEvent::Volume(
                    state.get_sink_icon_name().to_string(),
                    state.volume,
                    state.muted,
                ));
                last_volume.set(state.volume);
                last_muted.set(state.muted);
            }

            if state.mic_volume != last_mic_volume.get() || state.mic_muted != last_mic_muted.get()
            {
                OsdEventService::send_event(crate::services::osd::OsdEvent::Volume(
                    state.get_source_icon_name().to_string(),
                    state.mic_volume,
                    state.mic_muted,
                ));
                last_mic_volume.set(state.mic_volume);
                last_mic_muted.set(state.mic_muted);
            }
        });

        CapsLockService::subscribe_capslock(move |enabled| {
            OsdEventService::send_event(crate::services::osd::OsdEvent::CapsLock(enabled));
        });

        NumLockService::subscribe_numlock(move |enabled| {
            OsdEventService::send_event(crate::services::osd::OsdEvent::NumLock(enabled));
        });

        ClipboardService::subscribe_clipboard(move || {
            OsdEventService::send_event(crate::services::osd::OsdEvent::Clipboard);
        });
    });

    app.run();
}
