mod day_carousel;
mod month_carousel;
mod views;
mod week_carousel;

use crate::services::PinController;
use chrono::Local;
use day_carousel::create_day_carousel;
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use month_carousel::create_month_carousel;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use views::{dayview, monthview, weekview, ViewMode};
use week_carousel::create_week_carousel;

const ICON_RESERVE_SPACE: &str = "󰐃"; // Icon for reserving space
const ICON_RELEASE_SPACE: &str = "󰐄"; // Icon for releasing space

pub fn create_tasker_window(
    application: &gtk::Application,
) -> (gtk::ApplicationWindow, PinController) {
    // Créer la fenêtre
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Nwidgets Tasker")
        .default_width(500)
        .default_height(600)
        .build();
    window.add_css_class("tasker-window");

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
    main_box.add_css_class("tasker-main");

    // Carrousels
    let (day_carousel, day_on_date_changed, day_reset) = create_day_carousel();
    let (week_carousel, week_on_date_changed, week_reset) = create_week_carousel();
    let (month_carousel, month_on_date_changed, month_reset) = create_month_carousel();

    // État de la vue actuelle
    let current_view = Rc::new(RefCell::new(ViewMode::Day));

    // Container pour le carrousel (changera selon la vue)
    let carousel_container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Container pour le contenu de la vue
    let view_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    view_container.set_vexpand(true);

    // Header avec titre et bouton pin/unpin
    let (header, is_exclusive, toggle_button, title_label) = create_header(
        &window,
        Rc::clone(&current_view),
        view_container.clone(),
        carousel_container.clone(),
        day_reset.clone(),
        week_reset.clone(),
        month_reset.clone(),
        day_carousel.clone(),
        week_carousel.clone(),
        month_carousel.clone(),
    );
    main_box.append(&header);

    main_box.append(&carousel_container);

    // Configurer les callbacks pour mettre à jour le label
    let title_label_clone1 = title_label.clone();
    *day_on_date_changed.borrow_mut() = Some(Box::new(move |date| {
        let month_abbr = date.format("%b").to_string();
        let year_short = date.format("%y").to_string();
        let title_text = format!("{} {}", month_abbr, year_short);
        title_label_clone1.set_text(&title_text);
    }));

    let title_label_clone2 = title_label.clone();
    *week_on_date_changed.borrow_mut() = Some(Box::new(move |date| {
        let month_abbr = date.format("%b").to_string();
        let year_short = date.format("%y").to_string();
        let title_text = format!("{} {}", month_abbr, year_short);
        title_label_clone2.set_text(&title_text);
    }));

    let title_label_clone3 = title_label.clone();
    *month_on_date_changed.borrow_mut() = Some(Box::new(move |date| {
        let month_abbr = date.format("%b").to_string();
        let year_short = date.format("%y").to_string();
        let title_text = format!("{} {}", month_abbr, year_short);
        title_label_clone3.set_text(&title_text);
    }));

    // Ajouter le container de vue
    main_box.append(&view_container);

    // Zone d'ajout de tâche
    let (add_task_area, task_entry) = create_add_task_area();
    main_box.append(&add_task_area);

    // Initialiser avec la vue jour
    update_view(&view_container, ViewMode::Day);
    update_carousel(
        &carousel_container,
        ViewMode::Day,
        &day_carousel,
        &week_carousel,
        &month_carousel,
    );

    window.set_child(Some(&main_box));

    // Ajouter l'action toggle-tasker avec fermeture mutuelle
    let toggle_action = gtk::gio::SimpleAction::new("toggle-tasker", None);
    let window_clone = window.clone();
    let entry_clone = task_entry.clone();
    let app_clone = application.clone();
    toggle_action.connect_activate(move |_, _| {
        let is_visible = window_clone.is_visible();

        // Si on va ouvrir tasker, fermer control center s'il est ouvert
        if !is_visible {
            for window in app_clone.windows() {
                if window
                    .title()
                    .map_or(false, |t| t.contains("Control Center"))
                    && window.is_visible()
                {
                    if let Some(action) = app_clone.lookup_action("toggle-control-center") {
                        action.activate(None);
                    }
                    break;
                }
            }
        }

        window_clone.set_visible(!is_visible);

        // Si on ouvre la fenêtre, donner le focus à l'entrée de tâche
        if !is_visible {
            entry_clone.grab_focus();
            println!("[TASKER] Toggle tasker window: true (focus grabbed)");
        } else {
            println!("[TASKER] Toggle tasker window: false");
        }
    });

    application.add_action(&toggle_action);

    // Gestionnaire de raccourci clavier Meta+P pour pin/unpin
    let key_controller = gtk::EventControllerKey::new();
    let window_clone2 = window.clone();
    let is_exclusive_clone = Rc::clone(&is_exclusive);
    let toggle_button_clone2 = toggle_button.clone();

    key_controller.connect_key_pressed(move |_, keyval, _, modifiers| {
        // Escape pour fermer si pas pinné
        if keyval == gtk::gdk::Key::Escape && !is_exclusive_clone.get() {
            window_clone2.set_visible(false);
            println!("[TASKER] Window hidden (Escape pressed, not pinned)");
            return gtk::glib::Propagation::Stop;
        }

        // Meta+P (Super+P)
        if keyval == gtk::gdk::Key::p && modifiers.contains(gtk::gdk::ModifierType::SUPER_MASK) {
            if is_exclusive_clone.get() {
                window_clone2.set_exclusive_zone(0);
                is_exclusive_clone.set(false);
                toggle_button_clone2.set_label("󰐃"); // ICON_RESERVE_SPACE
                println!("[TASKER] Released exclusive space (Meta+P)");
            } else {
                window_clone2.auto_exclusive_zone_enable();
                is_exclusive_clone.set(true);
                toggle_button_clone2.set_label("󰐄"); // ICON_RELEASE_SPACE
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
    current_view: Rc<RefCell<ViewMode>>,
    view_container: gtk::Box,
    carousel_container: gtk::Box,
    day_reset: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    week_reset: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    month_reset: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    day_carousel: gtk::Box,
    week_carousel: gtk::Box,
    month_carousel: gtk::Box,
) -> (gtk::Box, Rc<Cell<bool>>, gtk::Button, gtk::Label) {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.add_css_class("tasker-header");
    header.set_margin_start(16);
    header.set_margin_end(16);
    header.set_margin_top(16);
    header.set_margin_bottom(16);

    // Icône et titre
    let icon_label = gtk::Label::new(Some("")); // Nerd font icon for tasks
    icon_label.add_css_class("tasker-icon");
    header.append(&icon_label);

    // Obtenir le mois en cours (abrégé) et l'année (2 derniers chiffres)
    let now = Local::now();
    let month_abbr = now.format("%b").to_string();
    let year_short = now.format("%y").to_string();
    let title_text = format!("{} {}", month_abbr, year_short);

    let title_label = gtk::Label::new(Some(&title_text));
    title_label.add_css_class("tasker-title");

    // Ajouter un gestionnaire de clic pour revenir à aujourd'hui
    title_label.set_cursor_from_name(Some("pointer"));
    let gesture = gtk::GestureClick::new();
    let current_view_for_click = Rc::clone(&current_view);
    gesture.connect_released(move |_, _, _, _| {
        let view = *current_view_for_click.borrow();
        match view {
            ViewMode::Day => {
                if let Some(reset_fn) = &*day_reset.borrow() {
                    reset_fn();
                }
            }
            ViewMode::Week => {
                if let Some(reset_fn) = &*week_reset.borrow() {
                    reset_fn();
                }
            }
            ViewMode::Month => {
                if let Some(reset_fn) = &*month_reset.borrow() {
                    reset_fn();
                }
            }
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
    view_button.add_css_class("tasker-view-button");

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
        update_carousel(
            &carousel_container,
            new_view,
            &day_carousel,
            &week_carousel,
            &month_carousel,
        );
    });

    header.append(&view_button);

    // Bouton toggle pin/unpin (réserver/libérer l'espace)
    let toggle_button = gtk::Button::with_label(ICON_RESERVE_SPACE);
    toggle_button.add_css_class("tasker-pin-button");

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

fn update_carousel(
    container: &gtk::Box,
    view_mode: ViewMode,
    day_carousel: &gtk::Box,
    week_carousel: &gtk::Box,
    month_carousel: &gtk::Box,
) {
    // Vider le container
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    // Ajouter le bon carrousel
    let carousel_widget = match view_mode {
        ViewMode::Day => day_carousel,
        ViewMode::Week => week_carousel,
        ViewMode::Month => month_carousel,
    };

    container.append(carousel_widget);
}

fn create_add_task_area() -> (gtk::Box, gtk::Entry) {
    let add_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    add_box.add_css_class("tasker-add-area");
    add_box.set_margin_start(16);
    add_box.set_margin_end(16);
    add_box.set_margin_top(8);
    add_box.set_margin_bottom(16);

    // Entry pour ajouter une tâche
    let entry = gtk::Entry::new();
    entry.add_css_class("tasker-entry");
    entry.set_placeholder_text(Some("Add a new task..."));
    entry.set_hexpand(true);
    add_box.append(&entry);

    // Bouton ajouter
    let add_btn = gtk::Button::new();
    add_btn.add_css_class("tasker-add-button");
    add_btn.set_label(""); // Plus icon
    add_box.append(&add_btn);

    (add_box, entry)
}
