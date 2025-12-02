use crate::services::notifications::{Notification, NotificationService};
use crate::utils::icons;
use gtk::prelude::*;
use gtk4 as gtk;

pub fn create_notifications_section(notifications_list: gtk::Box) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Notifications"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Scrolled window for notifications
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_min_content_height(200);
    scrolled.set_max_content_height(300);
    scrolled.add_css_class("notifications-scroll");

    // Load existing notifications from history
    reload_notification_history(&notifications_list);

    scrolled.set_child(Some(&notifications_list));
    section.append(&scrolled);

    section
}

pub fn add_notification_to_list(notifications_list: &gtk::Box, notification: Notification) {
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

pub fn reload_notification_history(notifications_list: &gtk::Box) {
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
