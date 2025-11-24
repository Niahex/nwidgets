use gtk4 as gtk;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::cell::Cell;
use std::rc::Rc;
use std::process::Command;

#[derive(Clone, Copy, Debug)]
enum PowerAction {
    Lock,
    Logout,
    Suspend,
    Reboot,
    Shutdown,
}

impl PowerAction {
    fn icon(&self) -> &'static str {
        match self {
            PowerAction::Lock => "󰌾",      // Lock icon
            PowerAction::Logout => "󰍃",    // Logout icon
            PowerAction::Suspend => "󰒲",   // Sleep icon
            PowerAction::Reboot => "󰜉",    // Reboot icon
            PowerAction::Shutdown => "󰐥", // Shutdown icon
        }
    }

    fn label(&self) -> &'static str {
        match self {
            PowerAction::Lock => "Lock",
            PowerAction::Logout => "Logout",
            PowerAction::Suspend => "Suspend",
            PowerAction::Reboot => "Reboot",
            PowerAction::Shutdown => "Shutdown",
        }
    }

    fn execute(&self) {
        println!("[POWER_MENU] Executing action: {:?}", self);

        let result = match self {
            PowerAction::Lock => {
                // Hyprland lock command
                Command::new("hyprlock").spawn()
            }
            PowerAction::Logout => {
                // Hyprland exit
                Command::new("hyprctl").args(&["dispatch", "exit"]).spawn()
            }
            PowerAction::Suspend => {
                // Systemd suspend
                Command::new("systemctl").args(&["suspend"]).spawn()
            }
            PowerAction::Reboot => {
                // Systemd reboot
                Command::new("systemctl").args(&["reboot"]).spawn()
            }
            PowerAction::Shutdown => {
                // Systemd shutdown
                Command::new("systemctl").args(&["poweroff"]).spawn()
            }
        };

        if let Err(e) = result {
            eprintln!("[POWER_MENU] Failed to execute action {:?}: {}", self, e);
        }
    }

    fn all() -> Vec<PowerAction> {
        vec![
            PowerAction::Lock,
            PowerAction::Logout,
            PowerAction::Suspend,
            PowerAction::Reboot,
            PowerAction::Shutdown,
        ]
    }
}

pub fn create_power_menu_window(application: &gtk::Application) -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Power Menu")
        .build();

    // Cacher par défaut
    window.set_visible(false);

    // Configuration Layer Shell - centré
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    // Container principal avec overlay semi-transparent
    let overlay_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    overlay_box.set_halign(gtk::Align::Fill);
    overlay_box.set_valign(gtk::Align::Fill);
    overlay_box.set_hexpand(true);
    overlay_box.set_vexpand(true);
    overlay_box.add_css_class("power-menu-overlay");

    // Container centré pour les boutons
    let center_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    center_box.set_halign(gtk::Align::Center);
    center_box.set_valign(gtk::Align::Center);
    center_box.set_hexpand(true);
    center_box.set_vexpand(true);

    // Titre
    let title_label = gtk::Label::new(Some("Power Menu"));
    title_label.set_margin_bottom(32);
    title_label.add_css_class("power-menu-title");
    center_box.append(&title_label);

    // Container horizontal pour les boutons
    let buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 24);
    buttons_box.set_halign(gtk::Align::Center);

    // Index du bouton sélectionné
    let selected_index = Rc::new(Cell::new(0));
    let actions = PowerAction::all();
    let mut buttons = Vec::new();

    // Créer les boutons
    for (i, action) in actions.iter().enumerate() {
        let button = create_power_button(*action);
        buttons_box.append(&button);
        buttons.push(button);

        // Sélectionner le premier bouton par défaut
        if i == 0 {
            update_button_selection(&buttons[i], true);
        }
    }

    center_box.append(&buttons_box);

    // Hint pour la navigation
    let hint_label = gtk::Label::new(Some("← → to navigate  •  Enter to execute  •  Escape to cancel"));
    hint_label.set_margin_top(32);
    hint_label.add_css_class("power-menu-hint");
    center_box.append(&hint_label);

    overlay_box.append(&center_box);
    window.set_child(Some(&overlay_box));

    // Gestion du clavier
    let key_controller = gtk::EventControllerKey::new();
    let window_clone = window.clone();
    let selected_index_clone = Rc::clone(&selected_index);
    let actions_clone = actions.clone();
    let buttons_clone = buttons.clone();

    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        let current = selected_index_clone.get();
        let total = actions_clone.len();

        match keyval {
            gtk::gdk::Key::Left => {
                // Désélectionner l'ancien
                update_button_selection(&buttons_clone[current], false);

                // Aller à gauche (avec wrap)
                let new_index = if current == 0 { total - 1 } else { current - 1 };
                selected_index_clone.set(new_index);

                // Sélectionner le nouveau
                update_button_selection(&buttons_clone[new_index], true);

                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Right => {
                // Désélectionner l'ancien
                update_button_selection(&buttons_clone[current], false);

                // Aller à droite (avec wrap)
                let new_index = (current + 1) % total;
                selected_index_clone.set(new_index);

                // Sélectionner le nouveau
                update_button_selection(&buttons_clone[new_index], true);

                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter => {
                // Exécuter l'action sélectionnée
                let action = actions_clone[current];
                window_clone.set_visible(false);
                action.execute();
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Escape => {
                // Fermer le menu
                window_clone.set_visible(false);
                gtk::glib::Propagation::Stop
            }
            _ => gtk::glib::Propagation::Proceed,
        }
    });

    window.add_controller(key_controller);

    // Action toggle
    let toggle_action = gtk::gio::SimpleAction::new("toggle-power-menu", None);
    let window_clone = window.clone();
    let selected_index_reset = Rc::clone(&selected_index);
    let buttons_reset = buttons.clone();

    toggle_action.connect_activate(move |_, _| {
        let is_visible = window_clone.is_visible();

        if !is_visible {
            // Reset selection au premier bouton
            let old_index = selected_index_reset.get();
            update_button_selection(&buttons_reset[old_index], false);
            selected_index_reset.set(0);
            update_button_selection(&buttons_reset[0], true);
        }

        window_clone.set_visible(!is_visible);
        println!("[POWER_MENU] Toggle power menu: {}", !is_visible);
    });

    application.add_action(&toggle_action);

    window
}

fn create_power_button(action: PowerAction) -> gtk::Box {
    let button_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    button_box.set_size_request(120, 140);
    button_box.add_css_class("power-button");

    // Icône
    let icon_label = gtk::Label::new(Some(action.icon()));
    icon_label.add_css_class("power-button-icon");
    button_box.append(&icon_label);

    // Label
    let text_label = gtk::Label::new(Some(action.label()));
    text_label.add_css_class("power-button-text");
    button_box.append(&text_label);

    button_box
}

fn update_button_selection(button_box: &gtk::Box, selected: bool) {
    if selected {
        button_box.add_css_class("power-button-selected");
        button_box.remove_css_class("power-button");
    } else {
        button_box.add_css_class("power-button");
        button_box.remove_css_class("power-button-selected");
    }
}
