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
    pub window_title: Option<String>,
    pub app_icon: Option<String>,
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

        let mut devices = Self::parse_devices(&output_str, "Sinks:");
        // Also get sinks from Filters section
        devices.extend(Self::parse_filters(&output_str, "Audio/Sink"));
        devices
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

        let mut devices = Self::parse_devices(&output_str, "Sources:");
        // Also get sources from Filters section
        devices.extend(Self::parse_filters(&output_str, "Audio/Source"));
        devices
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

    /// Parse devices from Filters section
    fn parse_filters(status: &str, device_type: &str) -> Vec<AudioDevice> {
        let mut devices = Vec::new();
        let mut in_filters = false;

        for line in status.lines() {
            let trimmed = line.trim();

            // Check if we're in the Filters section
            if trimmed.starts_with("├─ Filters:") {
                in_filters = true;
                continue;
            }

            // Exit Filters section
            if in_filters && (trimmed.starts_with("└─") || trimmed == "Video" || trimmed == "Settings") {
                break;
            }

            // Parse filter device lines
            if in_filters && trimmed.starts_with("│") {
                let content = trimmed.trim_start_matches("│").trim();

                // Check if this line contains our device type
                if content.contains(device_type) {
                    // Check if it's marked as default with *
                    let is_default = content.starts_with('*');
                    let content = content.trim_start_matches('*').trim();

                    // Format: "ID. Name [Type]"
                    let parts: Vec<&str> = content.splitn(2, '.').collect();
                    if parts.len() == 2 {
                        if let Ok(id) = parts[0].trim().parse::<u32>() {
                            let rest = parts[1].trim();

                            // Extract name (everything before [Type])
                            let name = if let Some(bracket) = rest.find('[') {
                                rest[..bracket].trim()
                            } else {
                                rest
                            };

                            devices.push(AudioDevice {
                                id,
                                name: name.to_string(),
                                description: name.to_string(),
                                is_default,
                            });
                        }
                    }
                }
            }
        }

        devices
    }

    fn parse_devices(status: &str, section: &str) -> Vec<AudioDevice> {
        let mut devices = Vec::new();
        let mut in_section = false;

        for line in status.lines() {
            let trimmed = line.trim();

            // Check if we're in the target section (e.g., "├─ Sinks:")
            if trimmed.starts_with("├─") && trimmed.contains(section) {
                in_section = true;
                continue;
            }

            // Exit section when we hit another section header or empty subsection
            if in_section {
                // If we hit another "├─" header, we're done with this section
                if trimmed.starts_with("├─") {
                    break;
                }

                // If we hit the Streams section or another top-level section, stop
                if trimmed.starts_with("└─") || trimmed == "Video" || trimmed == "Settings" {
                    break;
                }

                // Parse device line (starts with │)
                if trimmed.starts_with("│") {
                    // Skip empty lines (just "│  ")
                    let content = trimmed.trim_start_matches("│").trim();
                    if content.is_empty() {
                        break; // Empty line means end of devices in this section
                    }

                    if let Some(device) = Self::parse_device_line(trimmed) {
                        devices.push(device);
                    }
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

    fn parse_streams(status: &str, _section: &str) -> Vec<AudioStream> {
        let mut streams = Vec::new();
        let mut in_streams = false;

        for line in status.lines() {
            let trimmed = line.trim();

            // Check if we're in the Streams section under Audio
            if trimmed.starts_with("└─ Streams:") {
                in_streams = true;
                continue;
            }

            // Exit Streams section when we hit Video or Settings
            if in_streams && (trimmed == "Video" || trimmed == "Settings") {
                break;
            }

            // Parse stream line (simple format: "ID. App Name")
            // These lines don't start with tree characters in the Streams section
            if in_streams && !trimmed.is_empty() {
                // Skip lines that are sub-items (with deeper indentation or arrows)
                if trimmed.contains('>') || trimmed.starts_with("│") || trimmed.contains('<') {
                    continue;
                }

                // Count leading spaces to detect channel lines (which are more indented)
                // Main streams have ~8 spaces, channels have ~13+ spaces
                let leading_spaces = line.len() - line.trim_start().len();
                if leading_spaces > 10 {
                    continue; // This is a channel line, not a stream
                }

                if let Some(stream) = Self::parse_stream_line(trimmed) {
                    streams.push(stream);
                }
            }
        }

        streams
    }

    fn parse_stream_line(line: &str) -> Option<AudioStream> {
        let clean = line.trim();

        // Format: "ID. App Name" (simple format from Streams section)
        let parts: Vec<&str> = clean.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }

        let id = parts[0].trim().parse::<u32>().ok()?;
        let app_name = parts[1].trim().to_string();

        // Get volume info for this stream
        let (volume, muted) = match Command::new("wpctl")
            .args(&["get-volume", &id.to_string()])
            .output()
        {
            Ok(output) => {
                if let Ok(volume_str) = String::from_utf8(output.stdout) {
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
                    (volume, muted)
                } else {
                    (100, false)
                }
            }
            Err(_) => (100, false),
        };

        // Try to get additional metadata using pw-cli
        let (pw_app_name, window_title, app_icon, should_filter) = Self::get_stream_metadata(id);

        // Filter out input streams (microphone/record streams)
        if should_filter {
            return None;
        }

        // Use application.name from pw-cli if available, otherwise use wpctl name
        let final_app_name = pw_app_name.unwrap_or_else(|| app_name.clone());

        Some(AudioStream {
            id,
            name: final_app_name.clone(),
            app_name: final_app_name,
            volume,
            muted,
            window_title,
            app_icon,
        })
    }

    /// Get additional stream metadata using pw-cli
    fn get_stream_metadata(stream_id: u32) -> (Option<String>, Option<String>, Option<String>, bool) {
        let output = Command::new("pw-cli")
            .args(&["info", &stream_id.to_string()])
            .output();

        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                let mut window_title = None;
                let mut app_icon = None;
                let mut app_name = None;
                let mut process_id = None;
                let mut process_binary = None;
                let mut media_class = None;

                for line in output_str.lines() {
                    let trimmed = line.trim();

                    // Look for application.name
                    if trimmed.contains("application.name") {
                        if let Some(eq_pos) = trimmed.find('=') {
                            let value = trimmed[eq_pos + 1..].trim().trim_matches('"').trim();
                            if !value.is_empty() {
                                app_name = Some(value.to_string());
                            }
                        }
                    }

                    // Look for application.process.id
                    if trimmed.contains("application.process.id") {
                        if let Some(eq_pos) = trimmed.find('=') {
                            let value = trimmed[eq_pos + 1..].trim().trim_matches('"').trim();
                            process_id = Some(value.to_string());
                        }
                    }

                    // Look for application.process.binary
                    if trimmed.contains("application.process.binary") {
                        if let Some(eq_pos) = trimmed.find('=') {
                            let value = trimmed[eq_pos + 1..].trim().trim_matches('"').trim();
                            process_binary = Some(value.to_string());
                        }
                    }

                    // Look for media.class to detect input streams
                    if trimmed.contains("media.class") {
                        if let Some(eq_pos) = trimmed.find('=') {
                            let value = trimmed[eq_pos + 1..].trim().trim_matches('"').trim();
                            media_class = Some(value.to_string());
                        }
                    }

                    // Look for media.name or window.title
                    if trimmed.contains("media.name") || trimmed.contains("window.title") {
                        if let Some(eq_pos) = trimmed.find('=') {
                            let value = trimmed[eq_pos + 1..].trim().trim_matches('"').trim();
                            if !value.is_empty() {
                                window_title = Some(value.to_string());
                            }
                        }
                    }

                    // Look for application.icon-name
                    if trimmed.contains("application.icon-name") {
                        if let Some(eq_pos) = trimmed.find('=') {
                            let value = trimmed[eq_pos + 1..].trim().trim_matches('"').trim();
                            if !value.is_empty() {
                                app_icon = Some(value.to_string());
                            }
                        }
                    }
                }

                // Filter out input streams (record/microphone streams)
                if let Some(ref class) = media_class {
                    if class.contains("Stream/Input/Audio") {
                        return (None, None, None, true); // true = should be filtered
                    }
                }

                // Detect Discord/Vesktop from Electron apps
                if process_binary.as_deref() == Some("electron") {
                    if let Some(pid) = process_id {
                        if let Ok(cmdline) = std::fs::read_to_string(format!("/proc/{}/cmdline", pid)) {
                            if cmdline.contains("vesktop") || cmdline.contains("discord") {
                                app_name = Some("Discord".to_string());
                                app_icon = Some("discord".to_string());
                            }
                        }
                    }
                }

                return (app_name, window_title, app_icon, false);
            }
        }

        (None, None, None, false)
    }
}