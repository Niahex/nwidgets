use gtk4 as gtk;
use gtk::prelude::*;
use crate::services::bluetooth::BluetoothState;
use crate::theme::icons;

#[derive(Clone)]
pub struct BluetoothModule {
    pub container: gtk::Box,
    icon_label: gtk::Label,
    count_label: gtk::Label,
}

impl BluetoothModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 2); // ml-0.5 (2px)
        container.set_width_request(48);  // w-12 (48px)
        container.set_height_request(32); // h-8 (32px)
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon_label = gtk::Label::new(Some(icons::ICONS.bluetooth_off));
        icon_label.add_css_class("bluetooth-icon");
        icon_label.add_css_class("bluetooth-off");

        let count_label = gtk::Label::new(None);
        count_label.add_css_class("bluetooth-count");
        count_label.set_visible(false);

        container.append(&icon_label);
        container.append(&count_label);

        // Rendre le container cliquable
        let gesture = gtk::GestureClick::new();
        gesture.connect_released(move |_gesture, _n_press, _x, _y| {
            // Spawn un thread tokio pour toggle le bluetooth
            tokio::spawn(async {
                match crate::services::bluetooth::BluetoothService::toggle_power().await {
                    Ok(new_state) => {
                        println!("[BLUETOOTH] 🔵 Toggled to: {}", new_state);
                    }
                    Err(e) => {
                        println!("[BLUETOOTH] ❌ Failed to toggle: {:?}", e);
                    }
                }
            });
        });
        container.add_controller(gesture);

        Self {
            container,
            icon_label,
            count_label,
        }
    }

    pub fn update(&self, state: BluetoothState) {
        // Déterminer l'icône et la classe CSS selon l'état
        let (icon, css_class) = if !state.powered {
            (icons::ICONS.bluetooth_off, "bluetooth-off")
        } else if state.connected_devices > 0 {
            (icons::ICONS.bluetooth_connected, "bluetooth-connected")
        } else {
            (icons::ICONS.bluetooth_on, "bluetooth-on")
        };

        // Mettre à jour l'icône
        self.icon_label.set_text(icon);

        // Retirer toutes les classes de couleur
        self.icon_label.remove_css_class("bluetooth-off");
        self.icon_label.remove_css_class("bluetooth-connected");
        self.icon_label.remove_css_class("bluetooth-on");

        // Ajouter la classe appropriée
        self.icon_label.add_css_class(css_class);

        // Afficher le nombre d'appareils connectés si > 0
        if state.connected_devices > 0 {
            self.count_label.set_text(&state.connected_devices.to_string());
            self.count_label.set_visible(true);
        } else {
            self.count_label.set_visible(false);
        }
    }
}
