use super::audio_state::AudioState;
use crate::services::osd::{OsdEvent, OsdEventService};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioDevice {
    Sink,   // Output (speakers, headphones)
    Source, // Input (microphone)
}

impl AudioDevice {
    fn wpctl_target(&self) -> &str {
        match self {
            AudioDevice::Sink => "@DEFAULT_AUDIO_SINK@",
            AudioDevice::Source => "@DEFAULT_AUDIO_SOURCE@",
        }
    }

    fn get_icon_name(&self, state: &AudioState) -> &str {
        match self {
            AudioDevice::Sink => state.get_sink_icon_name(),
            AudioDevice::Source => state.get_source_icon_name(),
        }
    }

    fn is_device_muted(&self, state: &AudioState) -> bool {
        match self {
            AudioDevice::Sink => state.muted,
            AudioDevice::Source => state.mic_muted,
        }
    }
}

pub struct VolumeControl;

impl VolumeControl {
    // Generic methods
    fn get_device_volume(device: AudioDevice) -> u8 {
        if let Ok(output) = Command::new("wpctl")
            .args(["get-volume", device.wpctl_target()])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Some(volume_str) = output_str.split_whitespace().nth(1) {
                    if let Ok(volume) = volume_str.parse::<f32>() {
                        return (volume * 100.0) as u8;
                    }
                }
            }
        }
        0
    }

    fn is_device_muted(device: AudioDevice) -> bool {
        if let Ok(output) = Command::new("wpctl")
            .args(["get-volume", device.wpctl_target()])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                return output_str.contains("[MUTED]");
            }
        }
        false
    }

    fn set_device_volume(device: AudioDevice, volume: u8) {
        let volume_val = volume.min(100);
        let volume_str = format!("{volume_val}%");
        let _ = Command::new("wpctl")
            .args(["set-volume", device.wpctl_target(), &volume_str])
            .output();

        let state = Self::get_audio_state();
        let icon_name = device.get_icon_name(&state);
        let is_muted = device.is_device_muted(&state);
        OsdEventService::send_event(OsdEvent::Volume(
            icon_name.to_string(),
            volume_val,
            is_muted,
        ));
    }

    fn toggle_device_mute(device: AudioDevice) {
        let _ = Command::new("wpctl")
            .args(["set-mute", device.wpctl_target(), "toggle"])
            .output();

        let state = Self::get_audio_state();
        let icon_name = device.get_icon_name(&state);
        let is_muted = device.is_device_muted(&state);
        let volume = match device {
            AudioDevice::Sink => state.volume,
            AudioDevice::Source => state.mic_volume,
        };
        OsdEventService::send_event(OsdEvent::Volume(icon_name.to_string(), volume, is_muted));
    }

    // Public API for backward compatibility
    pub fn get_volume() -> u8 {
        Self::get_device_volume(AudioDevice::Sink)
    }

    pub fn is_muted() -> bool {
        Self::is_device_muted(AudioDevice::Sink)
    }

    pub fn get_mic_volume() -> u8 {
        Self::get_device_volume(AudioDevice::Source)
    }

    pub fn is_mic_muted() -> bool {
        Self::is_device_muted(AudioDevice::Source)
    }

    pub fn set_volume(volume: u8) {
        Self::set_device_volume(AudioDevice::Sink, volume);
    }

    pub fn set_mic_volume(volume: u8) {
        Self::set_device_volume(AudioDevice::Source, volume);
    }

    pub fn toggle_mute() {
        Self::toggle_device_mute(AudioDevice::Sink);
    }

    pub fn toggle_mic_mute() {
        Self::toggle_device_mute(AudioDevice::Source);
    }

    pub fn get_audio_state() -> AudioState {
        AudioState {
            volume: Self::get_volume(),
            muted: Self::is_muted(),
            mic_volume: Self::get_mic_volume(),
            mic_muted: Self::is_mic_muted(),
        }
    }
}
