mod audio_details;
mod bluetooth_details;
mod network_details;
mod notifications_details;
mod section_helpers;

use crate::services::notifications::NotificationService;
use crate::services::pipewire::{AudioState, PipeWireService};
use crate::utils::icons;
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use audio_details::{
    create_audio_section, setup_audio_section_callbacks, update_mic_details, update_volume_details,
    PanelManager,
};
use bluetooth_details::populate_bluetooth_details;
use network_details::populate_network_details;
use notifications_details::{add_notification_to_list, create_notifications_section};

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
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.add_css_class("control-center-container");

    // Create sections
    let (
        audio_section,
        volume_scale,
        mic_scale,
        volume_icon,
        mic_icon,
        volume_expanded,
        volume_expand_btn,
        mic_expanded,
        mic_expand_btn,
    ) = create_audio_section();
    container.append(&audio_section);

    // Combined Bluetooth/Network section
    let combined_section = gtk::Box::new(gtk::Orientation::Vertical, 0);
    combined_section.add_css_class("control-section");

    // Bluetooth and Network buttons on same line
    let bt_network_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    bt_network_box.set_hexpand(true);

    // Bluetooth button
    let bt_icon = icons::create_icon("bluetooth-active");
    bt_icon.set_size_request(24, 24);
    let bt_button = gtk::Button::new();
    bt_button.set_child(Some(&bt_icon));
    bt_button.add_css_class("section-button");
    bt_button.set_hexpand(true);

    // Network button
    let network_icon = icons::create_icon("network");
    network_icon.set_size_request(24, 24);
    let network_button = gtk::Button::new();
    network_button.set_child(Some(&network_icon));
    network_button.add_css_class("section-button");
    network_button.set_hexpand(true);

    bt_network_box.append(&bt_button);
    bt_network_box.append(&network_button);
    combined_section.append(&bt_network_box);

    // Shared expanded area (only one visible at a time)
    let shared_expanded = gtk::Box::new(gtk::Orientation::Vertical, 0);
    shared_expanded.add_css_class("expanded-section");
    shared_expanded.set_visible(false);
    combined_section.append(&shared_expanded);

    container.append(&combined_section);

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
        shared_expanded.clone(),
        shared_expanded.clone(), // Same for both bt and network
    );

    // Setup callbacks
    setup_audio_section_callbacks(
        &volume_expanded,
        &volume_expand_btn,
        &mic_expanded,
        &mic_expand_btn,
        &panels,
    );

    // Bluetooth button callback
    let shared_expanded_bt = shared_expanded.clone();
    let panels_bt = panels.clone();
    bt_button.connect_clicked(move |_| {
        panels_bt.collapse_all_except("bluetooth");

        // Clear and populate with bluetooth content
        while let Some(child) = shared_expanded_bt.first_child() {
            shared_expanded_bt.remove(&child);
        }
        populate_bluetooth_details(&shared_expanded_bt);
        shared_expanded_bt.set_visible(true);
    });

    // Network button callback
    let shared_expanded_net = shared_expanded.clone();
    let panels_net = panels.clone();
    network_button.connect_clicked(move |_| {
        panels_net.collapse_all_except("network");

        // Clear and populate with network content
        while let Some(child) = shared_expanded_net.first_child() {
            shared_expanded_net.remove(&child);
        }
        populate_network_details(&shared_expanded_net);
        shared_expanded_net.set_visible(true);
    });

    // Subscribe to audio updates
    let volume_expanded_sub = volume_expanded.clone();
    let mic_expanded_sub = mic_expanded.clone();
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

        // Update expanded details if visible
        update_volume_details(&volume_expanded_sub, &state);
        update_mic_details(&mic_expanded_sub, &state);
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
    let shared_expanded_clone = shared_expanded.clone();

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
                        while let Some(child) = shared_expanded_clone.first_child() {
                            shared_expanded_clone.remove(&child);
                        }
                        populate_bluetooth_details(&shared_expanded_clone);
                        shared_expanded_clone.set_visible(true);
                    }
                    "network" => {
                        panels_clone.collapse_all_except("network");
                        while let Some(child) = shared_expanded_clone.first_child() {
                            shared_expanded_clone.remove(&child);
                        }
                        populate_network_details(&shared_expanded_clone);
                        shared_expanded_clone.set_visible(true);
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

    // Subscribe to new notifications
    NotificationService::subscribe_notifications(move |notification| {
        add_notification_to_list(&notifications_list, notification);
    });

    window
}