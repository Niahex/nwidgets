use gtk4 as gtk;
use gtk::prelude::*;
use gtk::gio;
use crate::services::appmenu::{AppMenuService, AppMenuInfo};

#[derive(Clone)]
pub struct AppMenuModule {
    pub container: gtk::Box,
    menu_button: gtk::MenuButton,
    current_menu: std::rc::Rc<std::cell::RefCell<Option<AppMenuInfo>>>,
}

impl AppMenuModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add_css_class("appmenu-widget");

        // Créer le bouton de menu
        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        menu_button.set_tooltip_text(Some("Application Menu"));
        menu_button.add_css_class("appmenu-button");
        menu_button.set_visible(false); // Caché par défaut, visible uniquement si un menu est disponible

        container.append(&menu_button);

        let current_menu = std::rc::Rc::new(std::cell::RefCell::new(None));

        // Note: Pour l'instant on ne s'abonne pas automatiquement
        // Le module sera mis à jour via update_for_window() appelé depuis le panel principal

        Self {
            container,
            menu_button,
            current_menu,
        }
    }

    /// Mise à jour quand la fenêtre active change
    pub fn update_for_window(&self, window_address: &str) {
        // Convertir l'address Hyprland en window_id
        // Format: "0x..." en décimal
        let window_id = if let Some(hex_str) = window_address.strip_prefix("0x") {
            u32::from_str_radix(hex_str, 16).unwrap_or(0)
        } else {
            0
        };

        if window_id == 0 {
            log::debug!("Pas de fenêtre active valide");
            *self.current_menu.borrow_mut() = None;
            self.menu_button.set_visible(false);
            return;
        }

        if let Some(menu_info) = AppMenuService::get_menu_for_window(window_id) {
            log::info!("Mise à jour du menu pour fenêtre {} ({}): {}:{}",
                window_address, window_id, menu_info.service_name, menu_info.object_path);
            *self.current_menu.borrow_mut() = Some(menu_info.clone());

            // Créer et attacher le menu DBus
            self.load_menu_from_dbus(&menu_info);
        } else {
            log::debug!("Pas de menu pour la fenêtre {} ({})", window_address, window_id);
            *self.current_menu.borrow_mut() = None;
            self.menu_button.set_visible(false);
        }
    }

    /// Charge le menu depuis DBus
    fn load_menu_from_dbus(&self, menu_info: &AppMenuInfo) {
        let menu_button = self.menu_button.clone();
        let service_name = menu_info.service_name.clone();
        let object_path = menu_info.object_path.clone();

        // Charger le menu de manière asynchrone
        glib::MainContext::default().spawn_local(async move {
            match Self::create_dbus_menu(&service_name, &object_path).await {
                Ok(menu_model) => {
                    menu_button.set_menu_model(Some(&menu_model));
                    menu_button.set_visible(true);
                    log::info!("Menu DBus chargé avec succès");
                }
                Err(e) => {
                    log::warn!("Impossible de charger le menu DBus: {}", e);
                    menu_button.set_visible(false);
                }
            }
        });
    }

    /// Crée un MenuModel depuis un service DBus
    async fn create_dbus_menu(service_name: &str, object_path: &str) -> Result<gio::MenuModel, String> {
        // Obtenir une connexion DBus de manière synchrone
        let connection = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>)
            .map_err(|e| e.to_string())?;

        // Créer le MenuModel depuis DBus
        let model = gio::DBusMenuModel::get(
            &connection,
            Some(service_name),
            object_path,
        );

        log::debug!("MenuModel créé depuis DBus: {}:{}", service_name, object_path);
        Ok(model.upcast::<gio::MenuModel>())
    }
}
