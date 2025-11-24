use crate::services::systray::TrayItem;
use crate::theme::icons;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Clone)]
pub struct SystrayModule {
    pub container: gtk::CenterBox,
}

impl SystrayModule {
    pub fn new() -> Self {
        let container = gtk::CenterBox::new();
        container.add_css_class("systray-widget");
        container.set_width_request(35);
        container.set_height_request(50);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        Self { container }
    }

    pub fn update(&self, items: Vec<TrayItem>) {
        // Supprimer le widget central existant
        self.container.set_center_widget(None::<&gtk::Widget>);

        // Ajouter le premier item (ou rien si vide)
        if let Some(item) = items.first() {
            let icon = icons::ICONS.get_for_tray_item(&item.title, &item.id);

            let label = gtk::Label::new(Some(icon));
            label.set_halign(gtk::Align::Center);
            label.set_valign(gtk::Align::Center);
            label.add_css_class("systray-item");
            label.set_tooltip_text(Some(&format!("{} - {}", item.title, item.id)));

            self.container.set_center_widget(Some(&label));
        }
    }
}
