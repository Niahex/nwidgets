use super::audio_state::AudioDevice;
use std::process::Command;

pub struct DeviceManager;

impl DeviceManager {
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
            if in_filters
                && (trimmed.starts_with("└─") || trimmed == "Video" || trimmed == "Settings")
            {
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
        let content = line.trim_start_matches("│").trim();

        // Check if it's marked as default with *
        let is_default = content.starts_with('*');
        let content = content.trim_start_matches('*').trim();

        // Format: "ID. Name"
        let parts: Vec<&str> = content.splitn(2, '.').collect();
        if parts.len() == 2 {
            if let Ok(id) = parts[0].trim().parse::<u32>() {
                let name = parts[1].trim();
                return Some(AudioDevice {
                    id,
                    name: name.to_string(),
                    description: name.to_string(),
                    is_default,
                });
            }
        }
        None
    }
}
