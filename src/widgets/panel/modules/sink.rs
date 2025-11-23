use gtk4 as gtk;
use gtk::prelude::*;
use crate::services::pipewire::AudioState;
use crate::theme::icons;

#[derive(Clone)]
pub struct SinkModule {
    pub container: gtk::CenterBox,
    icon_label: gtk::Label,
}

impl SinkModule {
    pub fn new() -> Self {
        let container = gtk::CenterBox::new();
        container.add_css_class("sink-widget");
        container.set_width_request(50);
        container.set_height_request(50);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon_label = gtk::Label::new(Some(icons::ICONS.volume_high));
        icon_label.add_css_class("sink-icon");
        icon_label.set_halign(gtk::Align::Center);
        icon_label.set_valign(gtk::Align::Center);

        container.set_center_widget(Some(&icon_label));

        Self {
            container,
            icon_label,
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
    }
}
