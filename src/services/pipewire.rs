use crate::services::osd::{OsdEvent, OsdEventService};
use glib::MainContext;
use std::process::Command;
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AudioState {
    pub volume: u8,
    pub muted: bool,
    pub mic_volume: u8,
    pub mic_muted: bool,
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct AudioStream {
    pub id: u32,
    pub name: String,
    pub app_name: String,
    pub volume: u8,
    pub muted: bool,
}

impl AudioState {
    /// Retourne le nom de l'icône pour le sink (sortie audio) en fonction du volume et de l'état muted
    pub fn get_sink_icon_name(&self) -> &'static str {
        if self.muted {
            "sink-muted"
        } else if self.volume == 0 {
            "sink-zero"
        } else if self.volume < 33 {
            "sink-low"
        } else if self.volume < 66 {
            "sink-medium"
        } else {
            "sink-high"
        }
    }

    /// Retourne le nom de l'icône pour la source (entrée audio/micro) en fonction du volume et de l'état muted
    pub fn get_source_icon_name(&self) -> &'static str {
        if self.mic_muted {
            "source-muted"
        } else if self.mic_volume == 0 {
            "source-zero"
        } else if self.mic_volume < 33 {
            "source-low"
        } else if self.mic_volume < 66 {
            "source-medium"
        } else {
            "source-high"
        }
    }
}

pub struct PipeWireService;

impl PipeWireService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_volume(&self) -> u8 {
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

    pub fn is_muted(&self) -> bool {
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

    pub fn get_mic_volume(&self) -> u8 {
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

    pub fn is_mic_muted(&self) -> bool {
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

        let state = Self::get_audio_state();
        let icon_name = state.get_sink_icon_name();
        OsdEventService::send_event(OsdEvent::Volume(icon_name.to_string(), volume_val, state.muted));
    }

    pub fn set_mic_volume(volume: u8) {
        let volume_val = volume.min(100);
        let volume_str = format!("{}%", volume_val);
        let _ = Command::new("wpctl")
            .args(&["set-volume", "@DEFAULT_AUDIO_SOURCE@", &volume_str])
            .output();

        let state = Self::get_audio_state();
        let icon_name = state.get_source_icon_name();
        OsdEventService::send_event(OsdEvent::Volume(icon_name.to_string(), volume_val, state.mic_muted));
    }

    fn get_audio_state() -> AudioState {
        let service = Self::new();
        AudioState {
            volume: service.get_volume(),
            muted: service.is_muted(),
            mic_volume: service.get_mic_volume(),
            mic_muted: service.is_mic_muted(),
        }
    }

    pub fn subscribe_audio<F>(callback: F)
    where
        F: Fn(AudioState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let mut last_state = Self::get_audio_state();
            let _ = tx.send(last_state.clone());

            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let new_state = Self::get_audio_state();

                if new_state.volume != last_state.volume
                    || new_state.muted != last_state.muted
                    || new_state.mic_volume != last_state.mic_volume
                    || new_state.mic_muted != last_state.mic_muted
                {
                    if tx.send(new_state.clone()).is_err() {
                        break;
                    }
                    last_state = new_state;
                }
            }
        });

        let (async_tx, async_rx) = async_channel::unbounded();

        std::thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                if async_tx.send_blocking(state).is_err() {
                    break;
                }
            }
        });

        MainContext::default().spawn_local(async move {
            while let Ok(state) = async_rx.recv().await {
                callback(state);
            }
        });
    }

    /// List all audio output devices (sinks)
    pub fn list_sinks() -> Vec<AudioDevice> {
        let output = match Command::new("wpctl").args(&["status"]).output() {
            Ok(out) => out,
            Err(_) => return Vec::new(),
        };

        let output_str = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        Self::parse_devices(&output_str, "Sinks:")
    }

    /// List all audio input devices (sources)
    pub fn list_sources() -> Vec<AudioDevice> {
        let output = match Command::new("wpctl").args(&["status"]).output() {
            Ok(out) => out,
            Err(_) => return Vec::new(),
        };

        let output_str = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        Self::parse_devices(&output_str, "Sources:")
    }

    /// List all audio playback streams (sink-inputs)
    pub fn list_sink_inputs() -> Vec<AudioStream> {
        let output = match Command::new("wpctl").args(&["status"]).output() {
            Ok(out) => out,
            Err(_) => return Vec::new(),
        };

        let output_str = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        Self::parse_streams(&output_str, "Sink inputs:")
    }

    /// List all audio recording streams (source-outputs)
    pub fn list_source_outputs() -> Vec<AudioStream> {
        let output = match Command::new("wpctl").args(&["status"]).output() {
            Ok(out) => out,
            Err(_) => return Vec::new(),
        };

        let output_str = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        Self::parse_streams(&output_str, "Source outputs:")
    }

    /// Set volume for a specific stream (application)
    pub fn set_stream_volume(stream_id: u32, volume: u8) {
        let volume_val = volume.min(100);
        let volume_str = format!("{}%", volume_val);
        let _ = Command::new("wpctl")
            .args(&["set-volume", &stream_id.to_string(), &volume_str])
            .output();
    }

    /// Toggle mute for a specific stream
    pub fn toggle_stream_mute(stream_id: u32) {
        let _ = Command::new("wpctl")
            .args(&["set-mute", &stream_id.to_string(), "toggle"])
            .output();
    }

    /// Set default sink (output device)
    pub fn set_default_sink(sink_id: u32) {
        let _ = Command::new("wpctl")
            .args(&["set-default", &sink_id.to_string()])
            .output();
    }

    /// Set default source (input device)
    pub fn set_default_source(source_id: u32) {
        let _ = Command::new("wpctl")
            .args(&["set-default", &source_id.to_string()])
            .output();
    }

    fn parse_devices(status: &str, section: &str) -> Vec<AudioDevice> {
        let mut devices = Vec::new();
        let mut in_section = false;
        let mut in_audio = false;

        for line in status.lines() {
            let trimmed = line.trim();

            // Check if we're in the Audio section
            if trimmed == "Audio" {
                in_audio = true;
                continue;
            }

            // Exit Audio section if we hit another main section
            if in_audio && !trimmed.is_empty() && !trimmed.starts_with("│") && !trimmed.starts_with("├") && !trimmed.starts_with("└") {
                if trimmed != section {
                    in_audio = false;
                }
            }

            // Check if we're in the target section
            if in_audio && trimmed.starts_with(section) {
                in_section = true;
                continue;
            }

            // Exit section on next header or empty section indicator
            if in_section && (trimmed.is_empty() || (trimmed.starts_with("├") && !trimmed.contains("."))) {
                break;
            }

            // Parse device line
            if in_section && (trimmed.starts_with("│") || trimmed.starts_with("├") || trimmed.starts_with("└")) {
                if let Some(device) = Self::parse_device_line(trimmed) {
                    devices.push(device);
                }
            }
        }

        devices
    }

    fn parse_device_line(line: &str) -> Option<AudioDevice> {
        // Remove tree characters and whitespace
        let clean = line.trim_start_matches(&['│', '├', '└', '─', ' '][..]);

        // Check if this is a default device (marked with *)
        let is_default = clean.starts_with('*');
        let clean = clean.trim_start_matches('*').trim();

        // Format: "ID. Description [name]"
        let parts: Vec<&str> = clean.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }

        let id = parts[0].trim().parse::<u32>().ok()?;
        let rest = parts[1].trim();

        // Extract name from brackets if present
        let (description, name) = if let Some(bracket_start) = rest.rfind('[') {
            if let Some(bracket_end) = rest.rfind(']') {
                let desc = rest[..bracket_start].trim().to_string();
                let nam = rest[bracket_start + 1..bracket_end].trim().to_string();
                (desc, nam)
            } else {
                (rest.to_string(), String::new())
            }
        } else {
            (rest.to_string(), String::new())
        };

        Some(AudioDevice {
            id,
            name,
            description,
            is_default,
        })
    }

    fn parse_streams(status: &str, section: &str) -> Vec<AudioStream> {
        let mut streams = Vec::new();
        let mut in_section = false;
        let mut in_streams = false;

        for line in status.lines() {
            let trimmed = line.trim();

            // Check if we're in the Streams section
            if trimmed == "Streams:" {
                in_streams = true;
                continue;
            }

            // Exit Streams section
            if in_streams && !trimmed.is_empty() && !trimmed.starts_with("│") && !trimmed.starts_with("├") && !trimmed.starts_with("└") {
                in_streams = false;
            }

            // Check if we're in the target section
            if in_streams && trimmed.starts_with(section) {
                in_section = true;
                continue;
            }

            // Exit section
            if in_section && (trimmed.is_empty() || (trimmed.starts_with("├") && !trimmed.contains("."))) {
                break;
            }

            // Parse stream line
            if in_section && (trimmed.starts_with("│") || trimmed.starts_with("├") || trimmed.starts_with("└")) {
                if let Some(stream) = Self::parse_stream_line(trimmed) {
                    streams.push(stream);
                }
            }
        }

        streams
    }

    fn parse_stream_line(line: &str) -> Option<AudioStream> {
        // Remove tree characters
        let clean = line.trim_start_matches(&['│', '├', '└', '─', ' '][..]).trim();

        // Format: "ID. App Name: Stream Name [extra info]"
        let parts: Vec<&str> = clean.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }

        let id = parts[0].trim().parse::<u32>().ok()?;
        let rest = parts[1].trim();

        // Split on first colon to get app name and stream name
        let (app_name, name) = if let Some(colon_pos) = rest.find(':') {
            let app = rest[..colon_pos].trim();
            let stream = rest[colon_pos + 1..].trim();
            // Remove any bracketed info from stream name
            let stream = if let Some(bracket) = stream.find('[') {
                stream[..bracket].trim()
            } else {
                stream
            };
            (app.to_string(), stream.to_string())
        } else {
            (rest.to_string(), String::new())
        };

        // Get volume info for this stream
        let volume_output = Command::new("wpctl")
            .args(&["get-volume", &id.to_string()])
            .output()
            .ok()?;

        let volume_str = String::from_utf8(volume_output.stdout).ok()?;
        let muted = volume_str.contains("[MUTED]");

        let volume = if let Some(vol_str) = volume_str.split_whitespace().nth(1) {
            if let Ok(vol) = vol_str.parse::<f32>() {
                (vol * 100.0) as u8
            } else {
                100
            }
        } else {
            100
        };

        Some(AudioStream {
            id,
            name,
            app_name,
            volume,
            muted,
        })
    }
}