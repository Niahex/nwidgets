use crate::icons;
use crate::services::pipewire::AudioState;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Clone)]
pub struct SinkModule {
    pub container: gtk::CenterBox,
    icon: gtk::Image,
}

impl SinkModule {
    pub fn new() -> Self {
        let container = gtk::CenterBox::new();
        container.add_css_class("sink-widget");
        container.set_width_request(35);
        container.set_height_request(50);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon = icons::create_icon("audio-volume-high-panel", 22);
        icon.add_css_class("sink-icon");
        icon.set_halign(gtk::Align::Center);
        icon.set_valign(gtk::Align::Center);

        container.set_center_widget(Some(&icon));

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

        Self { container, icon }
    }

    pub fn update(&self, state: &AudioState) {
        let icon_name = if state.muted {
            "audio-volume-muted-panel"
        } else if state.volume < 33 {
            "audio-volume-low-panel"
        } else {
            "audio-volume-high-panel"
        };

        self.icon.set_icon_name(Some(icon_name));
    }
}
