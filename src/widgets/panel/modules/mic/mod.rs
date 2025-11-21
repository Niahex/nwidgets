use gtk4 as gtk;
use gtk::prelude::*;
use crate::services::pipewire::AudioState;
use crate::theme::icons;

#[derive(Clone)]
pub struct MicModule {
    pub container: gtk::Box,
    icon_label: gtk::Label,
}

impl MicModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        container.set_width_request(32);
        container.set_height_request(32);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon_label = gtk::Label::new(Some(icons::ICONS.microphone));
        icon_label.add_css_class("mic-icon");

        container.append(&icon_label);

        Self {
            container,
            icon_label,
        }
    }

    pub fn update(&self, state: &AudioState) {
        let icon = if state.mic_muted {
            icons::ICONS.microphone_slash
        } else {
            icons::ICONS.microphone
        };

        self.icon_label.set_text(icon);
    }
}
