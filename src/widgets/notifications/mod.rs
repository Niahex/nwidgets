use gtk4 as gtk;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell, KeyboardMode};
use crate::services::notifications::{Notification, NotificationService};
use crate::theme::colors::COLORS;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn create_notifications_window(application: &gtk::Application) -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .build();

    // Cacher la fenêtre au démarrage (avant init_layer_shell)
    window.set_visible(false);

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 10);
    window.set_margin(Edge::Right, 10);
    window.set_keyboard_mode(KeyboardMode::None);

    // Container principal pour toutes les notifications
    let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    container.set_width_request(380);
    container.set_halign(gtk::Align::End);
    container.set_valign(gtk::Align::Start);

    // Appliquer un CSS pour rendre le fond du container transparent
    let container_css = gtk::CssProvider::new();
    container_css.load_from_data("box { background-color: transparent; }");
    container.style_context().add_provider(
        &container_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.set_child(Some(&container));

    // Stocker les notifications avec leurs timestamps pour le nettoyage
    let notifications: Rc<RefCell<Vec<Notification>>> = Rc::new(RefCell::new(Vec::new()));

    // Fonction pour mettre à jour l'affichage
    let update_display = {
        let window = window.clone();
        let notifications = Rc::clone(&notifications);
        move || {
            let notifs = notifications.borrow();
            let container: gtk::Box = window.child().unwrap().downcast().unwrap();

            // Vider le container
            while let Some(child) = container.first_child() {
                container.remove(&child);
            }

            // Si aucune notification, cacher la fenêtre
            if notifs.is_empty() {
                window.set_visible(false);
                return;
            }

            // Afficher chaque notification
            for notification in notifs.iter() {
                let notif_widget = create_notification_widget(notification);
                container.append(&notif_widget);
            }

            // Montrer la fenêtre
            window.set_visible(true);
        }
    };

    // S'abonner aux notifications via le service
    let notifications_clone = Rc::clone(&notifications);
    let update_display_clone = update_display.clone();
    NotificationService::subscribe_notifications(move |notification| {
        println!("[NOTIF_GTK] 📢 Received notification: {} - {}",
                 notification.summary, notification.body);

        let mut notifs = notifications_clone.borrow_mut();
        notifs.push(notification);
        notifs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        notifs.truncate(10);
        drop(notifs);

        update_display_clone();
    });

    // Timer pour nettoyer les notifications expirées (toutes les secondes)
    let notifications_cleanup = Rc::clone(&notifications);
    let update_display_cleanup = update_display.clone();
    glib::timeout_add_seconds_local(1, move || {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut notifs = notifications_cleanup.borrow_mut();
        let old_count = notifs.len();
        notifs.retain(|n| now - n.timestamp < 5);

        if notifs.len() != old_count {
            println!("[NOTIF_GTK] 🗑️  Cleaned up notifications: {} -> {}",
                     old_count, notifs.len());
            drop(notifs);
            update_display_cleanup();
        }

        glib::ControlFlow::Continue
    });

    window
}

fn create_notification_widget(notification: &Notification) -> gtk::Widget {
    // Container principal (Box) avec background et bordure
    let container = gtk::Box::new(gtk::Orientation::Vertical, 8);
    container.set_width_request(380);
    container.set_margin_start(12);
    container.set_margin_end(12);
    container.set_margin_top(12);
    container.set_margin_bottom(12);

    // Déterminer la couleur de bordure selon l'urgence
    let border_color = match notification.urgency {
        2 => COLORS.red.to_hex_string(),      // Critical - rouge
        1 => COLORS.yellow.to_hex_string(),   // Normal - jaune
        _ => COLORS.frost1.to_hex_string(),   // Low - bleu
    };

    // Appliquer un style CSS inline pour le background et la bordure
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(&format!(
        "box {{
            background-color: {};
            border-left: 4px solid {};
            border-radius: 8px;
            padding: 12px;
        }}",
        COLORS.polar2.to_hex_string(),
        border_color
    ));

    container.style_context().add_provider(
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Header: titre + timestamp
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.set_hexpand(true);

    let title = gtk::Label::new(Some(&notification.summary));
    title.set_halign(gtk::Align::Start);
    title.set_hexpand(true);

    // Style pour le titre (couleur snow0, gras)
    let title_css = gtk::CssProvider::new();
    title_css.load_from_data(&format!(
        "label {{ color: {}; font-weight: bold; font-size: 14px; }}",
        COLORS.snow0.to_hex_string()
    ));
    title.style_context().add_provider(
        &title_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    header.append(&title);

    // Calculer le temps écoulé
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - notification.timestamp;

    let time_str = if elapsed < 60 {
        "now".to_string()
    } else if elapsed < 3600 {
        format!("{}m ago", elapsed / 60)
    } else {
        format!("{}h ago", elapsed / 3600)
    };

    let time_label = gtk::Label::new(Some(&time_str));
    time_label.set_halign(gtk::Align::End);

    // Style pour le timestamp (couleur polar3, petit)
    let time_css = gtk::CssProvider::new();
    time_css.load_from_data(&format!(
        "label {{ color: {}; font-size: 11px; }}",
        COLORS.polar3.to_hex_string()
    ));
    time_label.style_context().add_provider(
        &time_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    header.append(&time_label);

    // Ajouter le header au container interne
    let inner_container = gtk::Box::new(gtk::Orientation::Vertical, 8);
    inner_container.append(&header);

    // Body
    let body = gtk::Label::new(Some(&notification.body));
    body.set_halign(gtk::Align::Start);
    body.set_wrap(true);
    body.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    body.set_max_width_chars(50);

    // Style pour le body (couleur snow0, taille normale)
    let body_css = gtk::CssProvider::new();
    body_css.load_from_data(&format!(
        "label {{ color: {}; font-size: 12px; }}",
        COLORS.snow0.to_hex_string()
    ));
    body.style_context().add_provider(
        &body_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    inner_container.append(&body);

    // App name (si présent)
    if !notification.app_name.is_empty() {
        let app_label = gtk::Label::new(Some(&format!("from {}", notification.app_name)));
        app_label.set_halign(gtk::Align::Start);

        // Style pour l'app name (couleur polar3, petit, italique)
        let app_css = gtk::CssProvider::new();
        app_css.load_from_data(&format!(
            "label {{ color: {}; font-size: 11px; font-style: italic; }}",
            COLORS.polar3.to_hex_string()
        ));
        app_label.style_context().add_provider(
            &app_css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        inner_container.append(&app_label);
    }

    container.append(&inner_container);
    container.upcast()
}
