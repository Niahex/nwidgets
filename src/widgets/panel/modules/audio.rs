use super::base::{update_icon, PanelModuleConfig};
use crate::services::pipewire::AudioState;
use gtk4 as gtk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioDeviceType {
    Sink,   // Output device (speakers, headphones)
    Source, // Input device (microphone)
}

impl AudioDeviceType {
    fn widget_class(&self) -> &'static str {
        match self {
            AudioDeviceType::Sink => "sink-widget",
            AudioDeviceType::Source => "source-widget",
        }
    }

    fn icon_class(&self) -> &'static str {
        match self {
            AudioDeviceType::Sink => "sink-icon",
            AudioDeviceType::Source => "source-icon",
        }
    }

    fn default_icon(&self) -> &'static str {
        match self {
            AudioDeviceType::Sink => "sink-medium",
            AudioDeviceType::Source => "source-medium",
        }
    }

    fn action_parameter(&self) -> &'static str {
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
        let config = PanelModuleConfig::new(
            device_type.widget_class(),
            device_type.icon_class(),
            device_type.default_icon(),
            device_type.action_parameter(),
        );
        let (container, icon) = config.build();

        Self {
            container,
            icon,
            device_type,
        }
    }

    pub fn update(&self, state: &AudioState) {
        let icon_name = self.device_type.get_icon_name(state);
        update_icon(&self.icon, icon_name, Some(20));
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
