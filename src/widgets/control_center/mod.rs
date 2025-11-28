mod audio_details;
mod bluetooth_details;
mod network_details;
mod notifications_details;
mod quick_settings;
mod section_helpers;

use crate::utils::icons;
use crate::services::notifications::{Notification, NotificationService};
use crate::services::pipewire::{AudioState, PipeWireService};
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use audio_details::{create_audio_section, setup_audio_section_callbacks, setup_audio_updates, PanelManager};
use bluetooth_details::{create_bluetooth_section, setup_bluetooth_section_callbacks, setup_bluetooth_updates};
use network_details::{create_network_section, setup_network_section_callbacks, setup_network_updates};
use notifications_details::{create_notifications_section, add_notification_to_list};
use quick_settings::create_quick_settings_section;

pub fn create_control_center_window(application: &gtk::Application) -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Control Center")
        .default_width(500)
        .default_height(600)
        .build();
    window.add_css_class("control-center-window");

    // Layer Shell setup
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Right, true);
    window.set_keyboard_mode(KeyboardMode::OnDemand);

    // Main container
    let container = gtk::Box::new(gtk::Orientation::Vertical, 12);
    container.add_css_class("control-center-container");
    container.set_margin_start(16);
    container.set_margin_end(16);
    container.set_margin_top(16);
    container.set_margin_bottom(16);

    // Create sections
    let (audio_section, volume_scale, mic_scale, volume_icon, mic_icon, volume_expanded, volume_expand_btn, mic_expanded, mic_expand_btn) = create_audio_section();
    container.append(&audio_section);

    let (bluetooth_section, bt_expanded, bt_expand_btn) = create_bluetooth_section();
    container.append(&bluetooth_section);

    let (network_section, network_expanded, network_expand_btn) = create_network_section();
    container.append(&network_section);

    // Quick settings section
    let quick_settings = create_quick_settings_section();
    container.append(&quick_settings);

    // Notifications section
    let notifications_list = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let notifications_section = create_notifications_section(notifications_list.clone());
    container.append(&notifications_section);

    window.set_child(Some(&container));
    window.set_visible(false);

    // Setup panel manager for mutual exclusion
    let panels = PanelManager::new(
        volume_expanded.clone(),
        mic_expanded.clone(),
        bt_expanded.clone(),
        network_expanded.clone(),
    );

    // Setup callbacks
    setup_audio_section_callbacks(&volume_expanded, &volume_expand_btn, &mic_expanded, &mic_expand_btn, &panels);
    setup_bluetooth_section_callbacks(&bt_expanded, &bt_expand_btn, &panels);
    setup_network_section_callbacks(&network_expanded, &network_expand_btn, &panels);

    // Setup periodic updates
    setup_audio_updates(&volume_expanded, &mic_expanded);
    setup_bluetooth_updates(&bt_expanded);
    setup_network_updates(&network_expanded);

    // Subscribe to audio updates
    PipeWireService::subscribe_audio(move |state: AudioState| {
        volume_scale.set_value(state.volume as f64);
        mic_scale.set_value(state.mic_volume as f64);

        // Update icons dynamically
        if let Some(paintable) = icons::get_paintable(state.get_sink_icon_name()) {
            volume_icon.set_paintable(Some(&paintable));
        }
        if let Some(paintable) = icons::get_paintable(state.get_source_icon_name()) {
            mic_icon.set_paintable(Some(&paintable));
        }
    });

    // Action to toggle control center with optional section parameter
    let toggle_action = gtk::gio::SimpleAction::new_stateful(
        "toggle-control-center",
        Some(&String::static_variant_type()),
        &"".to_variant(),
    );
    let window_clone = window.clone();
    let panels_clone = panels.clone();
    let volume_expanded_clone = volume_expanded.clone();
    let mic_expanded_clone = mic_expanded.clone();
    let bt_expanded_clone = bt_expanded.clone();
    let network_expanded_clone = network_expanded.clone();

    toggle_action.connect_activate(move |_, param| {
        let is_visible = window_clone.is_visible();

        // Si la fenêtre est invisible, on l'ouvre
        if !is_visible {
            window_clone.set_visible(true);
            window_clone.present();

            // Si un paramètre de section est fourni, développer cette section
            if let Some(section) = param.and_then(|v| v.get::<String>()) {
                match section.as_str() {
                    "volume" | "sink" => {
                        panels_clone.collapse_all_except("volume");
                        volume_expanded_clone.set_visible(true);
                    }
                    "mic" | "source" => {
                        panels_clone.collapse_all_except("mic");
                        mic_expanded_clone.set_visible(true);
                    }
                    "bluetooth" => {
                        panels_clone.collapse_all_except("bluetooth");
                        bt_expanded_clone.set_visible(true);
                    }
                    "network" => {
                        panels_clone.collapse_all_except("network");
                        network_expanded_clone.set_visible(true);
                    }
                    _ => {}
                }
            }
        } else {
            // Si la fenêtre est visible, on la ferme
            window_clone.set_visible(false);
        }
    });

    application.add_action(&toggle_action);

    window
}
