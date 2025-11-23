mod services;
mod theme;
mod widgets;

// Include generated CSS from build.rs
mod style {
    include!(concat!(env!("OUT_DIR"), "/generated_style.rs"));
}

use crate::services::appmenu::AppMenuService;
use crate::services::bluetooth::BluetoothService;
use crate::services::capslock::CapsLockService;
use crate::services::clipboard::ClipboardService;
use crate::services::hyprland::HyprlandService;
use crate::services::numlock::NumLockService;
use crate::services::osd::OsdEventService;
use crate::services::pipewire::PipeWireService;
use crate::services::systray::SystemTrayService;
use crate::widgets::chat::create_chat_overlay;
use crate::widgets::launcher::create_launcher_window;
use crate::widgets::notifications::create_notifications_window;
use crate::widgets::osd::create_osd_window;
use crate::widgets::panel::create_panel_window;
use crate::widgets::power_menu::create_power_menu_window;
use crate::widgets::tasker::create_tasker_window;
use gtk4::{self as gtk, prelude::*, Application};

const APP_ID: &str = "github.niahex.nwidgets";

fn main() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(|app| {
        // Load CSS styles
        style::load_css();

        // Start AppMenu service for global menu support
        log::info!("[APPMENU] Starting AppMenu service...");
        AppMenuService::start();

        // Créer l'overlay de chat (caché par défaut, toggle avec l'action "toggle-chat")
        let chat_overlay = create_chat_overlay(app);
        let chat_pin_controller = chat_overlay.pin_controller.clone();

        // Créer la fenêtre de tasker (cachée par défaut, toggle avec l'action "toggle-tasker")
        // Retourne aussi les contrôles pour le pin
        let (tasker_window, tasker_pin_controller) = create_tasker_window(app);

        // Créer le power menu (caché par défaut, toggle avec l'action "toggle-power-menu")
        let _power_menu_window = create_power_menu_window(app);

        // Créer le launcher (caché par défaut, toggle avec l'action "toggle-launcher")
        let _launcher_window = create_launcher_window(app);

        // Action pour pin la fenêtre actuellement focus
        let pin_action = gtk::gio::SimpleAction::new("pin-focused-window", None);
        let chat_window_clone = chat_overlay.window.clone();
        let tasker_window_clone = tasker_window.clone();
        let chat_pin_clone = chat_pin_controller.clone();
        let tasker_pin_clone = tasker_pin_controller.clone();

        pin_action.connect_activate(move |_, _| {
            // Vérifier quelle fenêtre est visible et focus
            if chat_window_clone.is_visible() && chat_window_clone.is_active() {
                println!("[PIN] Chat window is focused, toggling pin");
                chat_pin_clone.toggle();
            } else if tasker_window_clone.is_visible() && tasker_window_clone.is_active() {
                println!("[PIN] Tasker window is focused, toggling pin");
                tasker_pin_clone.toggle();
            } else {
                println!("[PIN] No pinnable window is focused");
            }
        });

        app.add_action(&pin_action);

        let (
            panel_window,
            active_window_module,
            appmenu_module,
            workspaces_module,
            bluetooth_module,
            systray_module,
            volume_module,
            mic_module,
            _pomodoro_module,
        ) = create_panel_window(app);
        panel_window.present();

        let osd_window = create_osd_window(app);
        osd_window.present();

        // Créer la fenêtre de notifications (mais ne pas la présenter car elle est cachée au démarrage)
        let _notifications_window = create_notifications_window(app);

        // S'abonner aux mises à jour de la fenêtre active
        let active_window_module_clone = active_window_module.clone();
        let appmenu_module_clone = appmenu_module.clone();
        HyprlandService::subscribe_active_window(move |active_window| {
            active_window_module_clone.update(active_window.clone());

            // Mettre à jour le module appmenu avec l'address de la fenêtre
            if let Some(ref window) = active_window {
                log::debug!("[APPMENU] Active window changed: {} (address: {})", window.class, window.address);
                appmenu_module_clone.update_for_window(&window.address);
            } else {
                log::debug!("[APPMENU] No active window");
                appmenu_module_clone.update_for_window("");
            }
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
                OsdEventService::send_event(crate::services::osd::OsdEvent::Volume(
                    state.volume,
                    state.muted,
                ));
                last_volume.set(state.volume);
                last_muted.set(state.muted);
            }

            // Envoyer OSD micro si changement
            if state.mic_muted != last_mic_muted.get() {
                OsdEventService::send_event(crate::services::osd::OsdEvent::Microphone(
                    state.mic_muted,
                ));
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
