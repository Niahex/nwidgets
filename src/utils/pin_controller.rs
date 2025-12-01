use gtk4::{self as gtk};
use gtk4_layer_shell::LayerShell;
use std::cell::Cell;
use std::rc::Rc;
use crate::utils::icons;

/// Contrôleur pour gérer le pin/unpin d'une fenêtre depuis l'extérieur
#[derive(Clone)]
pub struct PinController {
    window: gtk::ApplicationWindow,
    is_exclusive: Rc<Cell<bool>>,
    pin_icon: gtk::Image,
}

impl PinController {
    pub fn new(
        window: gtk::ApplicationWindow,
        is_exclusive: Rc<Cell<bool>>,
        pin_icon: gtk::Image,
    ) -> Self {
        Self {
            window,
            is_exclusive,
            pin_icon,
        }
    }

    pub fn toggle(&self) {
        if self.is_exclusive.get() {
            // Unpin
            self.window.set_exclusive_zone(0);
            self.is_exclusive.set(false);
            if let Some(paintable) = icons::get_paintable("pin") {
                self.pin_icon.set_paintable(Some(&paintable));
            }
            println!("[PIN_CONTROLLER] Released exclusive space");
        } else {
            // Pin
            self.window.auto_exclusive_zone_enable();
            self.is_exclusive.set(true);
            if let Some(paintable) = icons::get_paintable("unpin") {
                self.pin_icon.set_paintable(Some(&paintable));
            }
            println!("[PIN_CONTROLLER] Reserved exclusive space");
        }
    }
}
