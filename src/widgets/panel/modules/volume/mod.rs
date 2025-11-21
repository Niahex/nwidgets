use gtk4 as gtk;
use gtk::prelude::*;
use crate::services::pipewire::AudioState;
use crate::theme::icons;

#[derive(Clone)]
pub struct VolumeModule {
    pub container: gtk::Box,
    icon_label: gtk::Label,
    percent_label: gtk::Label,
}

impl VolumeModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        container.set_width_request(64);
        container.set_height_request(32);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon_label = gtk::Label::new(Some(icons::ICONS.volume_high));
        icon_label.add_css_class("volume-icon");

        let percent_label = gtk::Label::new(Some("100%"));
        percent_label.add_css_class("volume-percent");

        container.append(&icon_label);
        container.append(&percent_label);

        Self {
            container,
            icon_label,
            percent_label,
        }
    }

    pub fn update(&self, state: &AudioState) {
        let icon = if state.muted {
            icons::ICONS.volume_mute
        } else if state.volume < 33 {
            icons::ICONS.volume_low
        } else {
            icons::ICONS.volume_high
        };

        self.icon_label.set_text(icon);
        self.percent_label.set_text(&format!("{}%", state.volume));
    }
}
