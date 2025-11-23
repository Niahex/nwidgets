use crate::services::bluetooth::BluetoothState;
use crate::theme::icons;
use gtk::prelude::*;
use gtk4::{self as gtk, CenterBox};

#[derive(Clone)]
pub struct BluetoothModule {
    pub container: gtk::CenterBox,
    icon_label: gtk::Label,
}

impl BluetoothModule {
    pub fn new() -> Self {
        let container = gtk::CenterBox::new();
        container.add_css_class("bluetooth-widget");
        container.set_width_request(50);
        container.set_height_request(50);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon_label = gtk::Label::new(Some(icons::ICONS.bluetooth_off));
        icon_label.add_css_class("bluetooth-icon");
        icon_label.add_css_class("bluetooth-off");
        icon_label.set_halign(gtk::Align::Center);
        icon_label.set_valign(gtk::Align::Center);

        container.set_center_widget(Some(&icon_label));

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
    }
}
