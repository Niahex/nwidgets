use chrono::Local;
use glib::ControlFlow;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Clone)]
pub struct DateTimeModule {
    pub container: gtk::Box,
    time_label: gtk::Label,
    date_label: gtk::Label,
}

impl DateTimeModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 2); // gap-0.5 (2px)
        container.add_css_class("datetime-widget");
        container.set_width_request(64); // w-16 (64px)
        container.set_height_request(45); // h-16 (64px)
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let time_label = gtk::Label::new(None);
        time_label.add_css_class("datetime-time");
        time_label.set_halign(gtk::Align::Center);

        let date_label = gtk::Label::new(None);
        date_label.add_css_class("datetime-date");
        date_label.set_halign(gtk::Align::Center);

        container.append(&time_label);
        container.append(&date_label);

        let module = Self {
            container,
            time_label,
            date_label,
        };

        // Mise à jour initiale
        module.update();

        // Mettre à jour toutes les secondes
        let module_clone = module.clone();
        glib::timeout_add_seconds_local(1, move || {
            module_clone.update();
            ControlFlow::Continue
        });

        module
    }

    fn update(&self) {
        let now = Local::now();
        self.time_label.set_text(&now.format("%H:%M").to_string());
        self.date_label
            .set_text(&now.format("%d/%m/%y").to_string());
    }
}
