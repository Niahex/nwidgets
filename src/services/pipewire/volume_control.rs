use crate::services::osd::{OsdEvent, OsdEventService};
use std::process::Command;
use super::audio_state::AudioState;

pub struct VolumeControl;

impl VolumeControl {
    pub fn get_volume() -> u8 {
        match Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            Ok(output) => {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(volume_str) = output_str.split_whitespace().nth(1) {
                        if let Ok(volume) = volume_str.parse::<f32>() {
                            return (volume * 100.0) as u8;
                        }
                    }
                }
            }
            Err(_) => {}
        }
        0
    }

    pub fn is_muted() -> bool {
        if let Ok(output) = Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                return output_str.contains("[MUTED]");
            }
        }
        false
    }

    pub fn get_mic_volume() -> u8 {
        match Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
            .output()
        {
            Ok(output) => {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(volume_str) = output_str.split_whitespace().nth(1) {
                        if let Ok(volume) = volume_str.parse::<f32>() {
                            return (volume * 100.0) as u8;
                        }
                    }
                }
            }
            Err(_) => {}
        }
        0
    }

    pub fn is_mic_muted() -> bool {
        if let Ok(output) = Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                return output_str.contains("[MUTED]");
            }
        }
        false
    }

    pub fn set_volume(volume: u8) {
        let volume_val = volume.min(100);
        let volume_str = format!("{}%", volume_val);
        let _ = Command::new("wpctl")
            .args(&["set-volume", "@DEFAULT_AUDIO_SINK@", &volume_str])
            .output();

        let state = AudioState {
            volume: Self::get_volume(),
            muted: Self::is_muted(),
            mic_volume: Self::get_mic_volume(),
            mic_muted: Self::is_mic_muted(),
        };
        let icon_name = state.get_sink_icon_name();
        OsdEventService::send_event(OsdEvent::Volume(icon_name.to_string(), volume_val, state.muted));
    }

    pub fn set_mic_volume(volume: u8) {
        let volume_val = volume.min(100);
        let volume_str = format!("{}%", volume_val);
        let _ = Command::new("wpctl")
            .args(&["set-volume", "@DEFAULT_AUDIO_SOURCE@", &volume_str])
            .output();

        let state = AudioState {
            volume: Self::get_volume(),
            muted: Self::is_muted(),
            mic_volume: Self::get_mic_volume(),
            mic_muted: Self::is_mic_muted(),
        };
        let icon_name = state.get_source_icon_name();
        OsdEventService::send_event(OsdEvent::Volume(icon_name.to_string(), volume_val, state.mic_muted));
    }

    pub fn toggle_mute() {
        let _ = Command::new("wpctl")
            .args(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
            .output();
    }

    pub fn toggle_mic_mute() {
        let _ = Command::new("wpctl")
            .args(&["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
            .output();
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
