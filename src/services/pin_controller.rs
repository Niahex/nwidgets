use gtk4::{self as gtk, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::cell::Cell;
use std::rc::Rc;

/// Contrôleur pour gérer le pin/unpin d'une fenêtre depuis l'extérieur
#[derive(Clone)]
pub struct PinController {
    window: gtk::ApplicationWindow,
    is_exclusive: Rc<Cell<bool>>,
    toggle_button: gtk::Button,
    icon_reserve: String,
    icon_release: String,
}

impl PinController {
    pub fn new(
        window: gtk::ApplicationWindow,
        is_exclusive: Rc<Cell<bool>>,
        toggle_button: gtk::Button,
        icon_reserve: &str,
        icon_release: &str,
    ) -> Self {
        Self {
            window,
            is_exclusive,
            toggle_button,
            icon_reserve: icon_reserve.to_string(),
            icon_release: icon_release.to_string(),
        }
    }

    pub fn toggle(&self) {
        if self.is_exclusive.get() {
            // Unpin
            self.window.set_exclusive_zone(0);
            self.is_exclusive.set(false);
            self.toggle_button.set_label(&self.icon_reserve);
            println!("[PIN_CONTROLLER] Released exclusive space");
        } else {
            // Pin
            self.window.auto_exclusive_zone_enable();
            self.is_exclusive.set(true);
            self.toggle_button.set_label(&self.icon_release);
            println!("[PIN_CONTROLLER] Reserved exclusive space");
        }
    }
}
