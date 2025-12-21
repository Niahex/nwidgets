use gpui::prelude::*;
use gpui::*;
use gpui::AsyncApp;
use std::fs;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    CapsLock,
    NumLock,
}

#[derive(Clone)]
pub struct LockStateChanged {
    pub lock_type: LockType,
    pub enabled: bool,
}

// On va utiliser le pattern Model de GPUI
pub struct LockMonitor {
    caps_lock: bool,
    num_lock: bool,
}

impl EventEmitter<LockStateChanged> for LockMonitor {}

impl LockMonitor {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let model = cx.new(|_cx| Self {
            caps_lock: Self::read_state(LockType::CapsLock),
            num_lock: Self::read_state(LockType::NumLock),
        });

        let weak_model = model.downgrade();

        cx.spawn(|cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor().timer(Duration::from_millis(100)).await;
                    
                    let current_caps = Self::read_state(LockType::CapsLock);
                    let current_num = Self::read_state(LockType::NumLock);

                    let _ = weak_model.update(&mut cx, |this, cx| {
                        if this.caps_lock != current_caps {
                            this.caps_lock = current_caps;
                            cx.emit(LockStateChanged {
                                lock_type: LockType::CapsLock,
                                enabled: current_caps,
                            });
                        }

                        if this.num_lock != current_num {
                            this.num_lock = current_num;
                            cx.emit(LockStateChanged {
                                lock_type: LockType::NumLock,
                                enabled: current_num,
                            });
                        }
                    });
                }
            }
        }).detach();

        model
    }

    fn read_state(lock_type: LockType) -> bool {
        // Essayer plusieurs chemins possibles pour capslock car input0 n'est pas garanti
        let paths = match lock_type {
            LockType::CapsLock => vec![
                "/sys/class/leds/input0::capslock/brightness",
                "/sys/class/leds/input1::capslock/brightness",
                "/sys/class/leds/input2::capslock/brightness",
                "/sys/class/leds/input3::capslock/brightness",
                "/sys/class/leds/capslock/brightness", // Parfois direct
            ],
            LockType::NumLock => vec![
                "/sys/class/leds/input0::numlock/brightness",
                "/sys/class/leds/input1::numlock/brightness",
                "/sys/class/leds/input2::numlock/brightness",
                "/sys/class/leds/input3::numlock/brightness",
                "/sys/class/leds/numlock/brightness",
            ],
        };

        for path in paths {
            if let Ok(content) = fs::read_to_string(path) {
                let trimmed = content.trim();
                return trimmed == "1" || trimmed.parse::<u8>().unwrap_or(0) > 0;
            }
        }
        
        false
    }
}