use gtk4 as gtk;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use crate::theme::colors::COLORS;
use crate::services::PinController;
use std::cell::Cell;
use std::rc::Rc;

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

    // Header avec titre et bouton pin/unpin (retourne aussi is_exclusive et toggle_button)
    let (header, is_exclusive, toggle_button) = create_header(&window);
    main_box.append(&header);

    // Zone de contenu pour les tâches
    let tasks_container = create_tasks_container();
    main_box.append(&tasks_container);

    // Zone d'ajout de tâche
    let add_task_area = create_add_task_area();
    main_box.append(&add_task_area);

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

fn create_header(window: &gtk::ApplicationWindow) -> (gtk::Box, Rc<Cell<bool>>, gtk::Button) {
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

    let title_label = gtk::Label::new(Some("Tasks"));
    let title_css = gtk::CssProvider::new();
    title_css.load_from_data(&format!(
        "label {{ color: {}; font-size: 20px; font-weight: bold; }}",
        COLORS.snow0.to_hex_string()
    ));
    title_label.style_context().add_provider(
        &title_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    header.append(&title_label);

    // Spacer
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header.append(&spacer);

    // Compteur de tâches
    let count_label = gtk::Label::new(Some("0 tasks"));
    let count_css = gtk::CssProvider::new();
    count_css.load_from_data(&format!(
        "label {{ color: {}; font-size: 12px; }}",
        COLORS.polar3.to_hex_string()
    ));
    count_label.style_context().add_provider(
        &count_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    header.append(&count_label);

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

    (header, is_exclusive, toggle_button)
}

fn create_tasks_container() -> gtk::ScrolledWindow {
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
    scrolled.set_vscrollbar_policy(gtk::PolicyType::Automatic);

    // Container pour les tâches
    let tasks_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    tasks_box.set_margin_start(16);
    tasks_box.set_margin_end(16);
    tasks_box.set_margin_top(16);
    tasks_box.set_margin_bottom(16);

    // Style du background
    let bg_css = gtk::CssProvider::new();
    bg_css.load_from_data(&format!(
        "box {{ background-color: {}; }}",
        COLORS.polar0.to_hex_string()
    ));
    tasks_box.style_context().add_provider(
        &bg_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Exemple de tâche (pour la démo)
    let sample_task = create_sample_task("Example task", false);
    tasks_box.append(&sample_task);

    scrolled.set_child(Some(&tasks_box));
    scrolled
}

fn create_sample_task(text: &str, completed: bool) -> gtk::Box {
    let task_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    task_box.set_margin_top(8);
    task_box.set_margin_bottom(8);
    task_box.set_margin_start(12);
    task_box.set_margin_end(12);

    // Style de la tâche
    let task_css = gtk::CssProvider::new();
    task_css.load_from_data(&format!(
        "box {{
            background-color: {};
            border-radius: 8px;
            padding: 12px;
        }}",
        COLORS.polar2.to_hex_string()
    ));
    task_box.style_context().add_provider(
        &task_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Checkbox
    let checkbox = gtk::CheckButton::new();
    checkbox.set_active(completed);
    task_box.append(&checkbox);

    // Texte de la tâche
    let task_label = gtk::Label::new(Some(text));
    task_label.set_halign(gtk::Align::Start);
    task_label.set_hexpand(true);

    let label_css = gtk::CssProvider::new();
    let color = if completed {
        COLORS.polar3.to_hex_string()
    } else {
        COLORS.snow0.to_hex_string()
    };
    let decoration = if completed { "line-through" } else { "none" };

    label_css.load_from_data(&format!(
        "label {{ color: {}; text-decoration: {}; }}",
        color, decoration
    ));
    task_label.style_context().add_provider(
        &label_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    task_box.append(&task_label);

    // Bouton supprimer
    let delete_btn = gtk::Button::new();
    delete_btn.set_label("");  // Trash icon
    let delete_css = gtk::CssProvider::new();
    delete_css.load_from_data(&format!(
        "button {{ color: {}; background: transparent; border: none; }}",
        COLORS.red.to_hex_string()
    ));
    delete_btn.style_context().add_provider(
        &delete_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    task_box.append(&delete_btn);

    task_box
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
