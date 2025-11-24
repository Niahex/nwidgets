use crate::icons;
use crate::services::pipewire::AudioState;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Clone)]
pub struct SourceModule {
    pub container: gtk::CenterBox,
    icon: gtk::Image,
}

impl SourceModule {
    pub fn new() -> Self {
        let container = gtk::CenterBox::new();
        container.add_css_class("source-widget");
        container.set_width_request(35);
        container.set_height_request(50);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon = icons::create_icon("audio-input-microphone-high-symbolic");
        icon.add_css_class("source-icon");
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
        let icon_name = if state.mic_muted {
            "audio-input-microphone-muted-symbolic."
        } else {
            "audio-input-microphone-high-symbolic"
        };

        self.icon.set_icon_name(Some(icon_name));
    }
}
