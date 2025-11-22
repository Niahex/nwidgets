mod day_carousel;
mod views;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use crate::theme::colors::COLORS;
use crate::services::PinController;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use day_carousel::create_day_carousel;
use chrono::Local;
use views::{ViewMode, dayview, weekview, monthview};

const ICON_RESERVE_SPACE: &str = "󰐃"; // Icon for reserving space
const ICON_RELEASE_SPACE: &str = "󰐄"; // Icon for releasing space

pub fn create_tasker_window(application: &gtk::Application) -> (gtk::ApplicationWindow, PinController) {
    // Créer la fenêtre
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Nwidgets Tasker")
        .default_width(500)
        .default_height(600)
        .build();

    // Cacher la fenêtre par défaut
    window.set_visible(false);

    // Configuration Layer Shell - ancré à droite
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Right, true);
    window.set_keyboard_mode(KeyboardMode::OnDemand);

    // Container principal
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Carrousel de jours
    let (day_carousel, on_date_changed, reset_to_today) = create_day_carousel();

    // État de la vue actuelle
    let current_view = Rc::new(RefCell::new(ViewMode::Day));

    // Container pour le contenu de la vue
    let view_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    view_container.set_vexpand(true);

    // Header avec titre et bouton pin/unpin (retourne aussi is_exclusive et toggle_button)
    let (header, is_exclusive, toggle_button, title_label) = create_header(&window, reset_to_today, Rc::clone(&current_view), view_container.clone());
    main_box.append(&header);

    main_box.append(&day_carousel);

    // Configurer le callback pour mettre à jour le label quand la date change
    *on_date_changed.borrow_mut() = Some(Box::new(move |date| {
        let month_abbr = date.format("%b").to_string();
        let year_short = date.format("%y").to_string();
        let title_text = format!("{} {}", month_abbr, year_short);
        title_label.set_text(&title_text);
    }));

    // Ajouter le container de vue
    main_box.append(&view_container);

    // Zone d'ajout de tâche
    let add_task_area = create_add_task_area();
    main_box.append(&add_task_area);

    // Initialiser avec la vue jour
    update_view(&view_container, ViewMode::Day);

    window.set_child(Some(&main_box));

    // Ajouter l'action toggle-tasker
    let toggle_action = gtk::gio::SimpleAction::new("toggle-tasker", None);
    let window_clone = window.clone();
    toggle_action.connect_activate(move |_, _| {
        let is_visible = window_clone.is_visible();
        window_clone.set_visible(!is_visible);
        println!("[TASKER] Toggle tasker window: {}", !is_visible);
    });

    application.add_action(&toggle_action);

    // Gestionnaire de raccourci clavier Meta+P pour pin/unpin
    let key_controller = gtk::EventControllerKey::new();
    let window_clone2 = window.clone();
    let is_exclusive_clone = Rc::clone(&is_exclusive);
    let toggle_button_clone2 = toggle_button.clone();

    key_controller.connect_key_pressed(move |_, keyval, _, modifiers| {
        // Meta+P (Super+P)
        if keyval == gtk::gdk::Key::p && modifiers.contains(gtk::gdk::ModifierType::SUPER_MASK) {
            if is_exclusive_clone.get() {
                window_clone2.set_exclusive_zone(0);
                is_exclusive_clone.set(false);
                toggle_button_clone2.set_label("󰐃");  // ICON_RESERVE_SPACE
                println!("[TASKER] Released exclusive space (Meta+P)");
            } else {
                window_clone2.auto_exclusive_zone_enable();
                is_exclusive_clone.set(true);
                toggle_button_clone2.set_label("󰐄");  // ICON_RELEASE_SPACE
                println!("[TASKER] Reserved exclusive space (Meta+P)");
            }
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    });

    window.add_controller(key_controller);

    // Créer le PinController pour permettre le contrôle externe
    let pin_controller = PinController::new(
        window.clone(),
        Rc::clone(&is_exclusive),
        toggle_button.clone(),
        ICON_RESERVE_SPACE,
        ICON_RELEASE_SPACE,
    );

    (window, pin_controller)
}


fn create_header(
    window: &gtk::ApplicationWindow,
    reset_to_today: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    current_view: Rc<RefCell<ViewMode>>,
    view_container: gtk::Box,
) -> (gtk::Box, Rc<Cell<bool>>, gtk::Button, gtk::Label) {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.set_margin_start(16);
    header.set_margin_end(16);
    header.set_margin_top(16);
    header.set_margin_bottom(16);

    // Appliquer un style au header
    let header_css = gtk::CssProvider::new();
    header_css.load_from_data(&format!(
        "box {{
            background-color: {};
            border-bottom: 2px solid {};
        }}",
        COLORS.polar1.to_hex_string(),
        COLORS.frost1.to_hex_string()
    ));
    header.style_context().add_provider(
        &header_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Icône et titre
    let icon_label = gtk::Label::new(Some(""));  // Nerd font icon for tasks
    let icon_css = gtk::CssProvider::new();
    icon_css.load_from_data(&format!(
        "label {{ color: {}; font-size: 24px; }}",
        COLORS.frost1.to_hex_string()
    ));
    icon_label.style_context().add_provider(
        &icon_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    header.append(&icon_label);

    // Obtenir le mois en cours (abrégé) et l'année (2 derniers chiffres)
    let now = Local::now();
    let month_abbr = now.format("%b").to_string();
    let year_short = now.format("%y").to_string();
    let title_text = format!("{} {}", month_abbr, year_short);

    let title_label = gtk::Label::new(Some(&title_text));
    let title_css = gtk::CssProvider::new();
    title_css.load_from_data(&format!(
        "label {{ color: {}; font-size: 20px; font-weight: bold; }}",
        COLORS.snow0.to_hex_string()
    ));
    title_label.style_context().add_provider(
        &title_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Ajouter un gestionnaire de clic pour revenir au jour actuel
    title_label.set_cursor_from_name(Some("pointer"));
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(move |_, _, _, _| {
        if let Some(reset_fn) = &*reset_to_today.borrow() {
            reset_fn();
        }
    });
    title_label.add_controller(gesture);

    header.append(&title_label);

    // Spacer
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header.append(&spacer);

    // Bouton pour changer de vue
    let view_mode = *current_view.borrow();
    let view_button = gtk::Button::with_label(view_mode.icon());
    let view_btn_css = gtk::CssProvider::new();
    view_btn_css.load_from_data(&format!(
        "button {{
            background-color: {};
            color: {};
            border: none;
            border-radius: 6px;
            padding: 8px 12px;
            font-size: 16px;
            margin-right: 8px;
        }}",
        COLORS.frost2.to_hex_string(),
        COLORS.polar0.to_hex_string()
    ));
    view_button.style_context().add_provider(
        &view_btn_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window_clone = window.clone();
    let view_button_clone = view_button.clone();
    view_button.connect_clicked(move |_| {
        let mut view = current_view.borrow_mut();
        *view = view.next();
        let new_view = *view;
        drop(view);

        view_button_clone.set_label(new_view.icon());

        let (width, height) = new_view.get_window_size();
        window_clone.set_default_size(width, height);

        update_view(&view_container, new_view);
    });

    header.append(&view_button);

    // Bouton toggle pin/unpin (réserver/libérer l'espace)
    let toggle_button = gtk::Button::with_label(ICON_RESERVE_SPACE);
    let toggle_css = gtk::CssProvider::new();
    toggle_css.load_from_data(&format!(
        "button {{
            background-color: {};
            color: {};
            border: none;
            border-radius: 6px;
            padding: 8px 12px;
            font-size: 16px;
        }}",
        COLORS.frost1.to_hex_string(),
        COLORS.polar0.to_hex_string()
    ));
    toggle_button.style_context().add_provider(
        &toggle_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let is_exclusive = Rc::new(Cell::new(false));
    let is_exclusive_for_button = Rc::clone(&is_exclusive);
    let window_clone = window.clone();
    let toggle_button_clone = toggle_button.clone();
    toggle_button.connect_clicked(move |_| {
        if is_exclusive_for_button.get() {
            window_clone.set_exclusive_zone(0);
            is_exclusive_for_button.set(false);
            toggle_button_clone.set_label(ICON_RESERVE_SPACE);
            println!("[TASKER] Released exclusive space");
        } else {
            window_clone.auto_exclusive_zone_enable();
            is_exclusive_for_button.set(true);
            toggle_button_clone.set_label(ICON_RELEASE_SPACE);
            println!("[TASKER] Reserved exclusive space");
        }
    });

    header.append(&toggle_button);

    (header, is_exclusive, toggle_button, title_label)
}

fn update_view(container: &gtk::Box, view_mode: ViewMode) {
    // Vider le container
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    // Ajouter la nouvelle vue
    let view_widget = match view_mode {
        ViewMode::Day => dayview::create_dayview().upcast::<gtk::Widget>(),
        ViewMode::Week => weekview::create_weekview().upcast::<gtk::Widget>(),
        ViewMode::Month => monthview::create_monthview().upcast::<gtk::Widget>(),
    };

    container.append(&view_widget);
}

fn create_add_task_area() -> gtk::Box {
    let add_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    add_box.set_margin_start(16);
    add_box.set_margin_end(16);
    add_box.set_margin_top(8);
    add_box.set_margin_bottom(16);

    // Style du background
    let bg_css = gtk::CssProvider::new();
    bg_css.load_from_data(&format!(
        "box {{
            background-color: {};
            border-top: 2px solid {};
            padding: 12px;
        }}",
        COLORS.polar1.to_hex_string(),
        COLORS.frost1.to_hex_string()
    ));
    add_box.style_context().add_provider(
        &bg_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Entry pour ajouter une tâche
    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("Add a new task..."));
    entry.set_hexpand(true);

    let entry_css = gtk::CssProvider::new();
    entry_css.load_from_data(&format!(
        "entry {{
            background-color: {};
            color: {};
            border: 1px solid {};
            border-radius: 6px;
            padding: 8px;
        }}",
        COLORS.polar2.to_hex_string(),
        COLORS.snow0.to_hex_string(),
        COLORS.polar3.to_hex_string()
    ));
    entry.style_context().add_provider(
        &entry_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    add_box.append(&entry);

    // Bouton ajouter
    let add_btn = gtk::Button::new();
    add_btn.set_label("");  // Plus icon
    let btn_css = gtk::CssProvider::new();
    btn_css.load_from_data(&format!(
        "button {{
            background-color: {};
            color: {};
            border: none;
            border-radius: 6px;
            padding: 8px 16px;
            font-size: 18px;
        }}",
        COLORS.frost1.to_hex_string(),
        COLORS.polar0.to_hex_string()
    ));
    add_btn.style_context().add_provider(
        &btn_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    add_box.append(&add_btn);

    add_box
}
