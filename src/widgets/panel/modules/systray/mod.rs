use gtk4 as gtk;
use gtk::prelude::*;
use crate::services::systray::TrayItem;
use crate::theme::icons;

#[derive(Clone)]
pub struct SystrayModule {
    pub container: gtk::Box,
}

impl SystrayModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 4); // gap-1 (4px)
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        Self { container }
    }

    pub fn update(&self, items: Vec<TrayItem>) {
        // Supprimer tous les enfants existants
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        // Ajouter les nouveaux items
        for item in items {
            let icon = icons::ICONS.get_for_tray_item(&item.title, &item.id);

            let label = gtk::Label::new(Some(icon));
            label.set_width_request(32);  // w-8 (32px)
            label.set_height_request(32); // h-8 (32px)
            label.set_halign(gtk::Align::Center);
            label.set_valign(gtk::Align::Center);
            label.add_css_class("systray-item");
            label.set_tooltip_text(Some(&format!("{} - {}", item.title, item.id)));

            self.container.append(&label);
        }
    }
}
