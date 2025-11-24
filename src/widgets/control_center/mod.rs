use gtk4 as gtk;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use crate::theme::icons;
use crate::services::notifications::{Notification, NotificationService};
use crate::services::pipewire::{AudioState, PipeWireService};

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

    // Audio section
    let (audio_section, volume_scale, mic_scale) = create_audio_section();
    container.append(&audio_section);

    // Bluetooth section
    let bluetooth_section = create_bluetooth_section();
    container.append(&bluetooth_section);

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
    toggle_action.connect_activate(move |_, _| {
        let is_visible = window_clone.is_visible();

        // Si on va ouvrir control center, fermer tasker s'il est ouvert
        if !is_visible {
            // Recharger l'historique des notifications avant d'ouvrir
            reload_notification_history(&notifications_list_for_toggle);

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

fn create_audio_section() -> (gtk::Box, gtk::Scale, gtk::Scale) {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Audio"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Volume controls
    let volume_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let volume_icon = gtk::Label::new(Some(icons::ICONS.volume_high));
    volume_icon.add_css_class("control-icon");
    let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.set_hexpand(true);
    volume_scale.add_css_class("control-scale");
    volume_box.append(&volume_icon);
    volume_box.append(&volume_scale);
    section.append(&volume_box);

    // Mic controls
    let mic_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let mic_icon = gtk::Label::new(Some(icons::ICONS.microphone));
    mic_icon.add_css_class("control-icon");
    let mic_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    mic_scale.set_hexpand(true);
    mic_scale.add_css_class("control-scale");
    mic_box.append(&mic_icon);
    mic_box.append(&mic_scale);
    section.append(&mic_box);

    (section, volume_scale, mic_scale)
}

fn create_bluetooth_section() -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Bluetooth"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    let bt_toggle = gtk::Switch::new();
    bt_toggle.add_css_class("control-switch");
    section.append(&bt_toggle);

    section
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

    println!("[CONTROL CENTER] Loading notification history: {} notifications", history.len());

    // Si pas d'historique, afficher un message vide
    if history.is_empty() {
        let empty_label = gtk::Label::new(Some("No notifications yet.\nNotifications will appear here when received."));
        empty_label.add_css_class("notification-empty");
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_valign(gtk::Align::Center);
        notifications_list.append(&empty_label);
    } else {
        // Afficher l'historique réel (les plus récentes sont déjà en premier dans le Vec)
        for notification in history {
            println!("[CONTROL CENTER] Adding notification from history: {} - {}", notification.summary, notification.body);
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
        if first_child.css_classes().contains(&"notification-empty".into()) {
            notifications_list.remove(&first_child);
        }
    }

    let notif_box = create_notification_widget(&notification);

    // Add to top of list (pour les nouvelles notifications)
    notifications_list.prepend(&notif_box);
}

fn add_notification_to_list_from_history(notifications_list: &gtk::Box, notification: Notification) {
    // Supprimer le message "No notifications" s'il existe
    if let Some(first_child) = notifications_list.first_child() {
        if first_child.css_classes().contains(&"notification-empty".into()) {
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
    println!("[CONTROL CENTER] Reloading notification history: {} notifications", history.len());

    if history.is_empty() {
        let empty_label = gtk::Label::new(Some("No notifications yet.\nNotifications will appear here when received."));
        empty_label.add_css_class("notification-empty");
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_valign(gtk::Align::Center);
        notifications_list.append(&empty_label);
    } else {
        // Afficher l'historique réel (les plus récentes sont déjà en premier dans le Vec)
        for notification in history {
            println!("[CONTROL CENTER] Adding notification from history: {} - {}", notification.summary, notification.body);
            let notif_box = create_notification_widget(&notification);
            notifications_list.append(&notif_box);
        }
    }
}

fn create_notification_widget(notification: &Notification) -> gtk::Box {
    let notif_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    notif_box.add_css_class("notification-item");

    let icon = gtk::Label::new(Some(icons::ICONS.info));
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
