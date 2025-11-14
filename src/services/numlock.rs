use std::fs;

pub struct NumLockService;

impl NumLockService {
    pub fn new() -> Self {
        Self
    }

    pub fn is_enabled(&self) -> bool {
        if let Ok(content) = fs::read_to_string("/sys/class/leds/input0::numlock/brightness") {
            return content.trim() == "1";
        }
        false
    }
}
