use std::process::Command;
use super::audio_state::AudioStream;

pub struct StreamManager;

impl StreamManager {
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

                // Detect Discord from various clients
                // 1. Vesktop (Electron-based)
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

                // 2. Official Discord client (uses .Discord-wrapped binary)
                if let Some(ref binary) = process_binary {
                    if binary.contains("Discord") {
                        app_name = Some("Discord".to_string());
                        app_icon = Some("discord".to_string());
                    }
                }

                // 3. WEBRTC VoiceEngine is Discord's audio engine
                if app_name.as_deref() == Some("WEBRTC VoiceEngine") {
                    app_name = Some("Discord".to_string());
                    app_icon = Some("discord".to_string());
                }

                return (app_name, window_title, app_icon, false);
            }
        }

        (None, None, None, false)
    }
}
