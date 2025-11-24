use chrono::Local;
use glib::ControlFlow;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Clone)]
pub struct DateTimeModule {
    pub container: gtk::CenterBox,
    datetime_box: gtk::Box,
    time_label: gtk::Label,
    date_label: gtk::Label,
}

impl DateTimeModule {
    pub fn new() -> Self {
        let container = gtk::CenterBox::new();
        container.add_css_class("datetime-widget");
        container.set_width_request(70);
        container.set_height_request(50);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        // Box interne pour les labels verticaux
        let datetime_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        datetime_box.set_halign(gtk::Align::Center);
        datetime_box.set_valign(gtk::Align::Center);

        let time_label = gtk::Label::new(None);
        time_label.add_css_class("datetime-time");
        time_label.set_halign(gtk::Align::Center);

        let date_label = gtk::Label::new(None);
        date_label.add_css_class("datetime-date");
        date_label.set_halign(gtk::Align::Center);

        datetime_box.append(&time_label);
        datetime_box.append(&date_label);
        container.set_center_widget(Some(&datetime_box));

        // Gestionnaire de clic pour ouvrir le centre de contrôle
        let gesture = gtk::GestureClick::new();
        gesture.connect_released(move |_, _, _, _| {
            if let Some(app) = gtk::gio::Application::default() {
                if let Some(action) = app.lookup_action("toggle-control-center") {
                    action.activate(None);
                }
            }
        });
        container.add_controller(gesture);

        let module = Self {
            container,
            datetime_box,
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
