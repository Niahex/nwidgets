use std::fs;

pub struct CapsLockService;

impl CapsLockService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_caps_lock_state() -> bool {
        if let Ok(content) = fs::read_to_string("/sys/class/leds/input0::capslock/brightness") {
            if let Ok(brightness) = content.trim().parse::<u8>() {
                return brightness > 0;
            }
        }
        false
    }

    pub fn is_enabled(&self) -> bool {
        Self::get_caps_lock_state()
    }
}
