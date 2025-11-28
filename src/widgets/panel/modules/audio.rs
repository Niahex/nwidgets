use crate::icons;
use crate::services::pipewire::AudioState;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioDeviceType {
    Sink,   // Output device (speakers, headphones)
    Source, // Input device (microphone)
}

impl AudioDeviceType {
    fn widget_class(&self) -> &str {
        match self {
            AudioDeviceType::Sink => "sink-widget",
            AudioDeviceType::Source => "source-widget",
        }
    }

    fn icon_class(&self) -> &str {
        match self {
            AudioDeviceType::Sink => "sink-icon",
            AudioDeviceType::Source => "source-icon",
        }
    }

    fn default_icon(&self) -> &str {
        match self {
            AudioDeviceType::Sink => "sink-medium",
            AudioDeviceType::Source => "source-medium",
        }
    }

    fn action_parameter(&self) -> &str {
        match self {
            AudioDeviceType::Sink => "sink",
            AudioDeviceType::Source => "source",
        }
    }

    fn get_icon_name(&self, state: &AudioState) -> &str {
        match self {
            AudioDeviceType::Sink => state.get_sink_icon_name(),
            AudioDeviceType::Source => state.get_source_icon_name(),
        }
    }
}

#[derive(Clone)]
pub struct AudioModule {
    pub container: gtk::CenterBox,
    icon: gtk::Image,
    device_type: AudioDeviceType,
}

impl AudioModule {
    pub fn new(device_type: AudioDeviceType) -> Self {
        let container = gtk::CenterBox::new();
        container.add_css_class(device_type.widget_class());
        container.set_width_request(35);
        container.set_height_request(50);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon = icons::create_icon_with_size(device_type.default_icon(), Some(20));
        icon.add_css_class(device_type.icon_class());
        icon.set_halign(gtk::Align::Center);
        icon.set_valign(gtk::Align::Center);

        container.set_center_widget(Some(&icon));

        // Click handler to open control center with appropriate section
        let action_param = device_type.action_parameter().to_string();
        let gesture = gtk::GestureClick::new();
        gesture.connect_released(move |_, _, _, _| {
            if let Some(app) = gtk::gio::Application::default() {
                if let Some(action) = app.lookup_action("toggle-control-center") {
                    action.activate(Some(&action_param.to_variant()));
                }
            }
        });
        container.add_controller(gesture);

        Self {
            container,
            icon,
            device_type,
        }
    }

    pub fn update(&self, state: &AudioState) {
        let icon_name = self.device_type.get_icon_name(state);

        if let Some(paintable) = icons::get_paintable_with_size(icon_name, Some(20)) {
            self.icon.set_paintable(Some(&paintable));
        }
    }
}

// Type aliases for backward compatibility
pub type SinkModule = AudioModule;
pub type SourceModule = AudioModule;

// Helper constructors for backward compatibility
impl AudioModule {
    pub fn new_sink() -> Self {
        Self::new(AudioDeviceType::Sink)
    }

    pub fn new_source() -> Self {
        Self::new(AudioDeviceType::Source)
    }
}
