use std::fs;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct NumLockService;

impl NumLockService {
    pub fn new() -> Self {
        println!("[NUMLOCK_SERVICE] ⌨️  Creating NumLockService");
        Self
    }

    /// Start monitoring NumLock state and send updates through the channel
    pub fn start_monitoring() -> mpsc::UnboundedReceiver<bool> {
        println!("[NUMLOCK_SERVICE] 🔍 Starting NumLock monitoring");
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let service = NumLockService::new();
            let mut last_state = service.is_enabled();
            println!("[NUMLOCK_SERVICE] 📊 Initial state: {}", last_state);

            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let current_state = service.is_enabled();

                if current_state != last_state {
                    println!("[NUMLOCK_SERVICE] 🔔 State changed: {} -> {}", last_state, current_state);

                    if tx.send(current_state).is_err() {
                        println!("[NUMLOCK_SERVICE] ⚠️  Receiver dropped, stopping monitoring");
                        break;
                    }

                    last_state = current_state;
                }
            }
        });

        rx
    }

    pub fn is_enabled(&self) -> bool {
        match fs::read_to_string("/sys/class/leds/input0::numlock/brightness") {
            Ok(content) => {
                let is_on = content.trim() == "1";
                println!("[NUMLOCK_SERVICE] 🔢 State: brightness='{}', is_on={}", content.trim(), is_on);
                is_on
            }
            Err(e) => {
                println!("[NUMLOCK_SERVICE] ❌ Failed to read state: {}", e);
                false
            }
        }
    }
}
