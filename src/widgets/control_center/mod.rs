mod audio_details;
mod bluetooth_details;
mod network_details;

use crate::icons;
use crate::services::notifications::{Notification, NotificationService};
use crate::services::pipewire::{AudioState, PipeWireService};
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use audio_details::{create_volume_details, create_mic_details, populate_volume_details, populate_mic_details};
use bluetooth_details::{create_bluetooth_details, populate_bluetooth_details};
use network_details::{create_network_details, populate_network_details};

pub fn create_control_center_window(application: &gtk::Application) -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Control Center")
        .default_width(500)
        .default_height(600)
        .build();
    window.add_css_class("control-center-window");

    // Layer Shell setup - même configuration que tasker
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

    // Audio section with expanded controls
    let (audio_section, volume_scale, mic_scale, volume_icon, mic_icon, volume_expanded, volume_expand_btn, mic_expanded, mic_expand_btn) = create_audio_section();
    container.append(&audio_section);

    // Bluetooth section
    let (bluetooth_section, bt_expanded, bt_expand_btn) = create_bluetooth_section();
    container.append(&bluetooth_section);

    // Network/VPN section
    let (network_section, network_expanded, network_expand_btn) = create_network_section();
    container.append(&network_section);

    // Create the shared expandable panels structure
    let panels = std::rc::Rc::new(ExpandablePanels {
        volume_panel: volume_expanded.clone(),
        volume_button: volume_expand_btn.clone(),
        mic_panel: mic_expanded.clone(),
        mic_button: mic_expand_btn.clone(),
        network_panel: network_expanded.clone(),
        network_button: network_expand_btn.clone(),
        bluetooth_panel: bt_expanded.clone(),
        bluetooth_button: bt_expand_btn.clone(),
    });

    // Setup mutual exclusion for volume expand button
    let panels_clone = panels.clone();
    let volume_expanded_clone = volume_expanded.clone();
    volume_expand_btn.connect_clicked(move |btn| {
        let is_visible = volume_expanded_clone.is_visible();
        if !is_visible {
            panels_clone.collapse_all_except("volume");
            volume_expanded_clone.set_visible(true);
            btn.set_icon_name("go-up-symbolic");
            populate_volume_details(&volume_expanded_clone);
        } else {
            volume_expanded_clone.set_visible(false);
            btn.set_icon_name("go-down-symbolic");
        }
    });

    // Setup mutual exclusion for mic expand button
    let panels_clone = panels.clone();
    let mic_expanded_clone = mic_expanded.clone();
    mic_expand_btn.connect_clicked(move |btn| {
        let is_visible = mic_expanded_clone.is_visible();
        if !is_visible {
            panels_clone.collapse_all_except("mic");
            mic_expanded_clone.set_visible(true);
            btn.set_icon_name("go-up-symbolic");
            populate_mic_details(&mic_expanded_clone);
        } else {
            mic_expanded_clone.set_visible(false);
            btn.set_icon_name("go-down-symbolic");
        }
    });

    // Setup mutual exclusion for network expand button
    let panels_clone = panels.clone();
    let network_expanded_clone = network_expanded.clone();
    network_expand_btn.connect_clicked(move |btn| {
        let is_visible = network_expanded_clone.is_visible();
        if !is_visible {
            panels_clone.collapse_all_except("network");
            network_expanded_clone.set_visible(true);
            btn.set_icon_name("go-up-symbolic");
            populate_network_details(&network_expanded_clone);
        } else {
            network_expanded_clone.set_visible(false);
            btn.set_icon_name("go-down-symbolic");
        }
    });

    // Setup mutual exclusion for bluetooth expand button
    let panels_clone = panels.clone();
    let bt_expanded_clone = bt_expanded.clone();
    bt_expand_btn.connect_clicked(move |btn| {
        let is_visible = bt_expanded_clone.is_visible();
        if !is_visible {
            panels_clone.collapse_all_except("bluetooth");
            bt_expanded_clone.set_visible(true);
            btn.set_icon_name("go-up-symbolic");
            populate_bluetooth_details(&bt_expanded_clone);
        } else {
            bt_expanded_clone.set_visible(false);
            btn.set_icon_name("go-down-symbolic");
        }
    });

    // Setup periodic updates (only update if visible)
    let volume_expanded_for_update = volume_expanded.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if volume_expanded_for_update.is_visible() {
            populate_volume_details(&volume_expanded_for_update);
        }
        gtk::glib::ControlFlow::Continue
    });

    let mic_expanded_for_update = mic_expanded.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if mic_expanded_for_update.is_visible() {
            populate_mic_details(&mic_expanded_for_update);
        }
        gtk::glib::ControlFlow::Continue
    });

    let network_expanded_for_update = network_expanded.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if network_expanded_for_update.is_visible() {
            populate_network_details(&network_expanded_for_update);
        }
        gtk::glib::ControlFlow::Continue
    });

    let bt_expanded_for_update = bt_expanded.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if bt_expanded_for_update.is_visible() {
            populate_bluetooth_details(&bt_expanded_for_update);
        }
        gtk::glib::ControlFlow::Continue
    });

    // Notifications history section
    let notifications_list = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let notifications_section = create_notifications_section(notifications_list.clone());
    container.append(&notifications_section);

    window.set_child(Some(&container));
    window.set_visible(false);

    // Subscribe to audio updates
    PipeWireService::subscribe_audio(move |state: AudioState| {
        volume_scale.set_value(state.volume as f64);
        mic_scale.set_value(state.mic_volume as f64);

        // Mettre à jour les icônes dynamiquement
        if let Some(paintable) = icons::get_paintable(state.get_sink_icon_name()) {
            volume_icon.set_paintable(Some(&paintable));
        }
        if let Some(paintable) = icons::get_paintable(state.get_source_icon_name()) {
            mic_icon.set_paintable(Some(&paintable));
        }
    });

    // Clone pour les différents usages
    let notifications_list_for_subscribe = notifications_list.clone();
    let notifications_list_for_toggle = notifications_list.clone();

    // Subscribe to notifications
    NotificationService::subscribe_notifications(move |notification: Notification| {
        add_notification_to_list(&notifications_list_for_subscribe, notification);
    });

    // Toggle action avec fermeture mutuelle
    let toggle_action = gtk::gio::SimpleAction::new("toggle-control-center", None);
    let window_clone = window.clone();
    let app_clone = application.clone();
    let panels_for_toggle = panels.clone();
    toggle_action.connect_activate(move |_, _| {
        let is_visible = window_clone.is_visible();

        // Si on va ouvrir control center, fermer tasker s'il est ouvert
        if !is_visible {
            // Recharger l'historique des notifications avant d'ouvrir
            reload_notification_history(&notifications_list_for_toggle);

            // Collapse all panels when opening control center
            panels_for_toggle.collapse_all_except("");

            for window in app_clone.windows() {
                if window.title().map_or(false, |t| t.contains("Tasker")) && window.is_visible() {
                    if let Some(action) = app_clone.lookup_action("toggle-tasker") {
                        action.activate(None);
                    }
                    break;
                }
            }
        }

        window_clone.set_visible(!is_visible);
        println!("[CONTROL CENTER] Toggle: {}", !is_visible);
    });
    application.add_action(&toggle_action);

    // Gestionnaire Escape pour fermer
    let key_controller = gtk::EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk::gdk::Key::Escape {
            window_clone.set_visible(false);
            println!("[CONTROL CENTER] Closed with Escape");
            gtk::glib::Propagation::Stop
        } else {
            gtk::glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    window
}

// Store all expandable panels to ensure only one is open at a time
struct ExpandablePanels {
    volume_panel: gtk::Box,
    volume_button: gtk::Button,
    mic_panel: gtk::Box,
    mic_button: gtk::Button,
    network_panel: gtk::Box,
    network_button: gtk::Button,
    bluetooth_panel: gtk::Box,
    bluetooth_button: gtk::Button,
}

impl ExpandablePanels {
    fn collapse_all_except(&self, keep_open: &str) {
        if keep_open != "volume" {
            self.volume_panel.set_visible(false);
            self.volume_button.set_icon_name("go-down-symbolic");
        }
        if keep_open != "mic" {
            self.mic_panel.set_visible(false);
            self.mic_button.set_icon_name("go-down-symbolic");
        }
        if keep_open != "network" {
            self.network_panel.set_visible(false);
            self.network_button.set_icon_name("go-down-symbolic");
        }
        if keep_open != "bluetooth" {
            self.bluetooth_panel.set_visible(false);
            self.bluetooth_button.set_icon_name("go-down-symbolic");
        }
    }
}

fn create_audio_section() -> (gtk::Box, gtk::Scale, gtk::Scale, gtk::Image, gtk::Image, gtk::Box, gtk::Button, gtk::Box, gtk::Button) {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Audio"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Volume controls with expand button
    let volume_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let volume_icon = icons::create_icon("sink-medium");
    volume_icon.add_css_class("control-icon");
    let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.set_hexpand(true);
    volume_scale.add_css_class("control-scale");
    volume_scale.set_draw_value(true);
    volume_scale.set_value_pos(gtk::PositionType::Right);

    // Connect volume change
    volume_scale.connect_value_changed(|scale| {
        let volume = scale.value() as u8;
        PipeWireService::set_volume(volume);
    });

    // Expand button for volume
    let volume_expand_btn = gtk::Button::from_icon_name("go-down-symbolic");
    volume_expand_btn.add_css_class("expand-button");

    volume_box.append(&volume_icon);
    volume_box.append(&volume_scale);
    volume_box.append(&volume_expand_btn);
    section.append(&volume_box);

    // Expanded box for volume details (initially hidden)
    let volume_expanded = create_volume_details();
    section.append(&volume_expanded);

    // Mic controls with expand button
    let mic_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let mic_icon = icons::create_icon("source-medium");
    mic_icon.add_css_class("control-icon");
    let mic_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    mic_scale.set_hexpand(true);
    mic_scale.add_css_class("control-scale");
    mic_scale.set_draw_value(true);
    mic_scale.set_value_pos(gtk::PositionType::Right);

    // Connect mic volume change
    mic_scale.connect_value_changed(|scale| {
        let volume = scale.value() as u8;
        PipeWireService::set_mic_volume(volume);
    });

    // Expand button for mic
    let mic_expand_btn = gtk::Button::from_icon_name("go-down-symbolic");
    mic_expand_btn.add_css_class("expand-button");

    mic_box.append(&mic_icon);
    mic_box.append(&mic_scale);
    mic_box.append(&mic_expand_btn);
    section.append(&mic_box);

    // Expanded box for mic details (initially hidden)
    let mic_expanded = create_mic_details();
    section.append(&mic_expanded);

    (section, volume_scale, mic_scale, volume_icon, mic_icon, volume_expanded, volume_expand_btn, mic_expanded, mic_expand_btn)
}

fn create_network_section() -> (gtk::Box, gtk::Box, gtk::Button) {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Network"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Network info with expand button
    let network_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    // Network status label
    let network_label = gtk::Label::new(Some("VPN Connections"));
    network_label.set_halign(gtk::Align::Start);
    network_label.set_hexpand(true);
    network_box.append(&network_label);

    // Expand button for network
    let network_expand_btn = gtk::Button::from_icon_name("go-down-symbolic");
    network_expand_btn.add_css_class("expand-button");
    network_box.append(&network_expand_btn);

    section.append(&network_box);

    // Expanded box for network details (initially hidden)
    let network_expanded = create_network_details();
    section.append(&network_expanded);

    (section, network_expanded, network_expand_btn)
}

fn create_bluetooth_section() -> (gtk::Box, gtk::Box, gtk::Button) {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Bluetooth"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Bluetooth toggle with expand button
    let bt_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let bt_toggle = gtk::Switch::new();
    bt_toggle.add_css_class("control-switch");
    bt_toggle.set_hexpand(true);

    // Expand button for bluetooth
    let bt_expand_btn = gtk::Button::from_icon_name("go-down-symbolic");
    bt_expand_btn.add_css_class("expand-button");

    bt_box.append(&bt_toggle);
    bt_box.append(&bt_expand_btn);
    section.append(&bt_box);

    // Expanded box for bluetooth details (initially hidden)
    let bt_expanded = create_bluetooth_details();
    section.append(&bt_expanded);

    (section, bt_expanded, bt_expand_btn)
}

fn create_notifications_section(notifications_list: gtk::Box) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");
    section.set_vexpand(true);

    let title = gtk::Label::new(Some("Notifications"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Scrolled window for notifications
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_vexpand(true);
    scrolled.add_css_class("notifications-scroll");

    notifications_list.add_css_class("notifications-list");

    // Charger l'historique existant
    let history = NotificationService::get_history();

    println!(
        "[CONTROL CENTER] Loading notification history: {} notifications",
        history.len()
    );

    // Si pas d'historique, afficher un message vide
    if history.is_empty() {
        let empty_label = gtk::Label::new(Some(
            "No notifications yet.\nNotifications will appear here when received.",
        ));
        empty_label.add_css_class("notification-empty");
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_valign(gtk::Align::Center);
        notifications_list.append(&empty_label);
    } else {
        // Afficher l'historique réel (les plus récentes sont déjà en premier dans le Vec)
        for notification in history {
            println!(
                "[CONTROL CENTER] Adding notification from history: {} - {}",
                notification.summary, notification.body
            );
            add_notification_to_list_from_history(&notifications_list, notification);
        }
    }

    scrolled.set_child(Some(&notifications_list));
    section.append(&scrolled);

    section
}

fn add_notification_to_list(notifications_list: &gtk::Box, notification: Notification) {
    // Supprimer le message "No notifications" s'il existe
    if let Some(first_child) = notifications_list.first_child() {
        if first_child
            .css_classes()
            .contains(&"notification-empty".into())
        {
            notifications_list.remove(&first_child);
        }
    }

    let notif_box = create_notification_widget(&notification);

    // Add to top of list (pour les nouvelles notifications)
    notifications_list.prepend(&notif_box);
}

fn add_notification_to_list_from_history(
    notifications_list: &gtk::Box,
    notification: Notification,
) {
    // Supprimer le message "No notifications" s'il existe
    if let Some(first_child) = notifications_list.first_child() {
        if first_child
            .css_classes()
            .contains(&"notification-empty".into())
        {
            notifications_list.remove(&first_child);
        }
    }

    let notif_box = create_notification_widget(&notification);

    // Add to bottom of list (pour l'historique qui est déjà dans l'ordre)
    notifications_list.append(&notif_box);
}

fn reload_notification_history(notifications_list: &gtk::Box) {
    // Supprimer tous les widgets existants
    while let Some(child) = notifications_list.first_child() {
        notifications_list.remove(&child);
    }

    // Recharger l'historique
    let history = NotificationService::get_history();
    println!(
        "[CONTROL CENTER] Reloading notification history: {} notifications",
        history.len()
    );

    if history.is_empty() {
        let empty_label = gtk::Label::new(Some(
            "No notifications yet.\nNotifications will appear here when received.",
        ));
        empty_label.add_css_class("notification-empty");
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_valign(gtk::Align::Center);
        notifications_list.append(&empty_label);
    } else {
        // Afficher l'historique réel (les plus récentes sont déjà en premier dans le Vec)
        for notification in history {
            println!(
                "[CONTROL CENTER] Adding notification from history: {} - {}",
                notification.summary, notification.body
            );
            let notif_box = create_notification_widget(&notification);
            notifications_list.append(&notif_box);
        }
    }
}

fn create_notification_widget(notification: &Notification) -> gtk::Box {
    let notif_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    notif_box.add_css_class("notification-item");

    let icon = icons::create_icon("dialog-information");
    icon.add_css_class("notification-icon");

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let title = gtk::Label::new(Some(&notification.summary));
    title.add_css_class("notification-title");
    title.set_halign(gtk::Align::Start);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let body = gtk::Label::new(Some(&notification.body));
    body.add_css_class("notification-body");
    body.set_halign(gtk::Align::Start);
    body.set_ellipsize(gtk::pango::EllipsizeMode::End);
    body.set_wrap(true);
    body.set_lines(2);

    content_box.append(&title);
    content_box.append(&body);
    content_box.set_hexpand(true);

    notif_box.append(&icon);
    notif_box.append(&content_box);

    notif_box
}
