use crate::widgets::r#macro::types::*;
use anyhow::Result;

pub fn get_input_devices() -> Result<Vec<String>> {
    let mut devices = Vec::new();
    let paths = ["/dev/input/by-id"];

    for base_path in paths {
        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let path_str = path.to_string_lossy();
                if (path_str.contains("event-kbd") || path_str.contains("event-mouse"))
                    && std::fs::metadata(&path).is_ok()
                {
                    if let Ok(canonical) = std::fs::canonicalize(&path) {
                        devices.push(canonical.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Ok(devices)
}

pub fn parse_event_line(line: &str) -> Option<ActionType> {
    if !line.contains("EV_KEY") || !line.contains("value") {
        return None;
    }

    let is_press = line.contains("value 1");
    let is_release = line.contains("value 0");

    if !is_press && !is_release {
        return None;
    }

    if line.contains("BTN_LEFT") {
        return Some(ActionType::MouseClick(MacroMouseButton::Left));
    } else if line.contains("BTN_RIGHT") {
        return Some(ActionType::MouseClick(MacroMouseButton::Right));
    } else if line.contains("BTN_MIDDLE") {
        return Some(ActionType::MouseClick(MacroMouseButton::Middle));
    } else if line.contains("KEY_") {
        if let Some(code_str) = line.split("code").nth(1) {
            if let Some(code) = code_str
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u32>().ok())
            {
                return if is_press {
                    Some(ActionType::KeyPress(code))
                } else {
                    Some(ActionType::KeyRelease(code))
                };
            }
        }
    }

    None
}
