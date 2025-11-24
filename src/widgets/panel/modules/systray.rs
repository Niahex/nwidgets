use crate::icons;
use crate::services::systray::TrayItem;
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
            let icon_name = match item.title.to_lowercase().as_str() {
                s if s.contains("discord") => "discord",
                s if s.contains("firefox") => "firefox",
                s if s.contains("chrome") => "google-chrome",
                s if s.contains("steam") => "steam",
                _ => "application-default-icon",
            };

            let icon = icons::create_icon(icon_name, 24);
            icon.set_halign(gtk::Align::Center);
            icon.set_valign(gtk::Align::Center);
            icon.add_css_class("systray-item");
            icon.set_tooltip_text(Some(&format!("{} - {}", item.title, item.id)));

            self.container.set_center_widget(Some(&icon));
        }
    }
}
