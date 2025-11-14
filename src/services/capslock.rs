use std::fs;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct CapsLockService;

impl CapsLockService {
    pub fn new() -> Self {
        println!("[CAPSLOCK_SERVICE] ⌨️  Creating CapsLockService");
        Self
    }

    /// Start monitoring CapsLock state and send updates through the channel
    pub fn start_monitoring() -> mpsc::UnboundedReceiver<bool> {
        println!("[CAPSLOCK_SERVICE] 🔍 Starting CapsLock monitoring");
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let service = CapsLockService::new();
            let mut last_state = service.is_enabled();
            println!("[CAPSLOCK_SERVICE] 📊 Initial state: {}", last_state);

            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let current_state = service.is_enabled();

                if current_state != last_state {
                    println!("[CAPSLOCK_SERVICE] 🔔 State changed: {} -> {}", last_state, current_state);

                    if tx.send(current_state).is_err() {
                        println!("[CAPSLOCK_SERVICE] ⚠️  Receiver dropped, stopping monitoring");
                        break;
                    }

                    last_state = current_state;
                }
            }
        });

        rx
    }

    pub fn get_caps_lock_state() -> bool {
        match fs::read_to_string("/sys/class/leds/input0::capslock/brightness") {
            Ok(content) => {
                match content.trim().parse::<u8>() {
                    Ok(brightness) => {
                        let is_on = brightness > 0;
                        println!("[CAPSLOCK_SERVICE] 🔑 State: brightness={}, is_on={}", brightness, is_on);
                        is_on
                    }
                    Err(e) => {
                        println!("[CAPSLOCK_SERVICE] ❌ Failed to parse brightness: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                println!("[CAPSLOCK_SERVICE] ❌ Failed to read state: {}", e);
                false
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        Self::get_caps_lock_state()
    }
}
