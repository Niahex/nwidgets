use gpui::prelude::*;
use gpui::AsyncApp;
use gpui::*;
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
                // Timer un peu plus lent pour économiser le CPU (200ms est assez réactif pour une LED)
                let mut interval = cx.background_executor().timer(Duration::from_millis(200));

                // Pré-calculer les chemins valides pour éviter de scanner à chaque itération
                let caps_paths = Self::find_valid_paths(LockType::CapsLock);
                let num_paths = Self::find_valid_paths(LockType::NumLock);

                loop {
                    interval.await;
                    interval = cx.background_executor().timer(Duration::from_millis(200));

                    let current_caps = Self::check_paths(&caps_paths);
                    let current_num = Self::check_paths(&num_paths);

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
        })
        .detach();

        model
    }

    fn find_valid_paths(lock_type: LockType) -> Vec<std::path::PathBuf> {
        let mut valid_paths = Vec::new();
        let pattern = match lock_type {
            LockType::CapsLock => "capslock",
            LockType::NumLock => "numlock",
        };

        if let Ok(entries) = fs::read_dir("/sys/class/leds") {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().contains(pattern) {
                    valid_paths.push(entry.path().join("brightness"));
                }
            }
        }
        valid_paths
    }

    fn check_paths(paths: &[std::path::PathBuf]) -> bool {
        for path in paths {
            if let Ok(content) = fs::read_to_string(path) {
                let trimmed = content.trim();
                if trimmed == "1" || trimmed.parse::<u8>().unwrap_or(0) > 0 {
                    return true;
                }
            }
        }
        false
    }

    // Méthode de fallback pour l'initialisation (avant le spawn du thread)
    fn read_state(lock_type: LockType) -> bool {
        let paths = Self::find_valid_paths(lock_type);
        Self::check_paths(&paths)
    }
}