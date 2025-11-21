mod theme;
mod widgets;
mod services;

use crate::widgets::chat::create_chat_window;
use crate::widgets::panel::create_panel_window;
use crate::widgets::osd::create_osd_window;
use crate::services::hyprland::HyprlandService;
use crate::services::bluetooth::BluetoothService;
use crate::services::systray::SystemTrayService;
use crate::services::pipewire::PipeWireService;
use crate::services::capslock::CapsLockService;
use crate::services::numlock::NumLockService;
use crate::services::clipboard::ClipboardService;
use crate::services::osd::OsdEventService;
use gtk4::{prelude::*, Application};

const APP_ID: &str = "com.nwidgets";

fn main() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(|app| {
        let chat_window = create_chat_window(app);
        chat_window.present();

        let (panel_window, active_window_module, workspaces_module, bluetooth_module, systray_module, volume_module, mic_module, _pomodoro_module) = create_panel_window(app);
        panel_window.present();

        let osd_window = create_osd_window(app);
        osd_window.present();

        // S'abonner aux mises à jour de la fenêtre active
        let active_window_module_clone = active_window_module.clone();
        HyprlandService::subscribe_active_window(move |active_window| {
            active_window_module_clone.update(active_window);
        });

        // S'abonner aux mises à jour des workspaces
        let workspaces_module_clone = workspaces_module.clone();
        HyprlandService::subscribe_workspace(move |workspaces, active_workspace| {
            workspaces_module_clone.update(workspaces, active_workspace);
        });

        // S'abonner aux mises à jour du bluetooth
        let bluetooth_module_clone = bluetooth_module.clone();
        BluetoothService::subscribe_bluetooth(move |state| {
            bluetooth_module_clone.update(state);
        });

        // S'abonner aux mises à jour du systray
        let systray_module_clone = systray_module.clone();
        SystemTrayService::subscribe_systray(move |items| {
            systray_module_clone.update(items);
        });

        // S'abonner aux mises à jour audio (volume + mic)
        let volume_module_clone = volume_module.clone();
        let mic_module_clone = mic_module.clone();
        let last_volume = std::cell::Cell::new(0u8);
        let last_muted = std::cell::Cell::new(false);
        let last_mic_muted = std::cell::Cell::new(false);
        PipeWireService::subscribe_audio(move |state| {
            volume_module_clone.update(&state);
            mic_module_clone.update(&state);

            // Envoyer OSD volume si changement
            if state.volume != last_volume.get() || state.muted != last_muted.get() {
                OsdEventService::send_event(crate::services::osd::OsdEvent::Volume(state.volume, state.muted));
                last_volume.set(state.volume);
                last_muted.set(state.muted);
            }

            // Envoyer OSD micro si changement
            if state.mic_muted != last_mic_muted.get() {
                OsdEventService::send_event(crate::services::osd::OsdEvent::Microphone(state.mic_muted));
                last_mic_muted.set(state.mic_muted);
            }
        });

        // S'abonner aux changements CapsLock
        CapsLockService::subscribe_capslock(move |enabled| {
            OsdEventService::send_event(crate::services::osd::OsdEvent::CapsLock(enabled));
        });

        // S'abonner aux changements NumLock
        NumLockService::subscribe_numlock(move |enabled| {
            OsdEventService::send_event(crate::services::osd::OsdEvent::NumLock(enabled));
        });

        // S'abonner aux changements du clipboard
        ClipboardService::subscribe_clipboard(move || {
            OsdEventService::send_event(crate::services::osd::OsdEvent::Clipboard);
        });
    });

    // Run the application
    app.run();
}