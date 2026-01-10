use gpui::prelude::*;
use gpui::*;
use inotify::{Inotify, WatchMask};
use tokio::io::unix::AsyncFd;
use std::path::PathBuf;

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
        let model = cx.new(|cx| {
            let this = Self {
                caps_lock: Self::read_state(LockType::CapsLock),
                num_lock: Self::read_state(LockType::NumLock),
            };

            cx.spawn(|weak_model: WeakEntity<Self>, mut cx| async move {
                eprintln!("[LockMonitor] Démarrage du monitoring inotify");
                
                let Ok(inotify) = Inotify::init() else {
                    eprintln!("[LockMonitor] Erreur: impossible d'initialiser inotify");
                    return;
                };

                let caps_path = Self::find_led_path(LockType::CapsLock);
                let num_path = Self::find_led_path(LockType::NumLock);

                if let Some(path) = &caps_path {
                    if inotify.watches().add(path, WatchMask::MODIFY).is_ok() {
                        eprintln!("[LockMonitor] Surveillance CapsLock: {:?}", path);
                    }
                }

                if let Some(path) = &num_path {
                    if inotify.watches().add(path, WatchMask::MODIFY).is_ok() {
                        eprintln!("[LockMonitor] Surveillance NumLock: {:?}", path);
                    }
                }

                let Ok(mut async_inotify) = AsyncFd::new(inotify) else {
                    eprintln!("[LockMonitor] Erreur: impossible de créer AsyncFd");
                    return;
                };

                let mut buffer = [0u8; 4096];
                loop {
                    let mut guard = match async_inotify.readable_mut().await {
                        Ok(g) => g,
                        Err(_) => break,
                    };

                    match guard.try_io(|inner| inner.get_mut().read_events(&mut buffer)) {
                        Ok(Ok(_events)) => {
                            eprintln!("[LockMonitor] Événement inotify détecté");
                            
                            let current_caps = Self::read_state(LockType::CapsLock);
                            let current_num = Self::read_state(LockType::NumLock);

                            eprintln!("[LockMonitor] État: CapsLock={}, NumLock={}", current_caps, current_num);

                            let _ = weak_model.update(&mut cx, |this, cx| {
                                if this.caps_lock != current_caps {
                                    eprintln!("[LockMonitor] CapsLock changé: {} -> {}", this.caps_lock, current_caps);
                                    this.caps_lock = current_caps;
                                    cx.emit(LockStateChanged {
                                        lock_type: LockType::CapsLock,
                                        enabled: current_caps,
                                    });
                                }

                                if this.num_lock != current_num {
                                    eprintln!("[LockMonitor] NumLock changé: {} -> {}", this.num_lock, current_num);
                                    this.num_lock = current_num;
                                    cx.emit(LockStateChanged {
                                        lock_type: LockType::NumLock,
                                        enabled: current_num,
                                    });
                                }
                            });
                        }
                        _ => {}
                    }

                    guard.clear_ready();
                }
            })
            .detach();

            this
        });

        model
    }

    fn find_led_path(lock_type: LockType) -> Option<PathBuf> {
        let pattern = match lock_type {
            LockType::CapsLock => "capslock",
            LockType::NumLock => "numlock",
        };

        std::fs::read_dir("/sys/class/leds")
            .ok()?
            .flatten()
            .find(|e| e.file_name().to_string_lossy().contains(pattern))
            .map(|e| e.path().join("brightness"))
    }

    fn read_state(lock_type: LockType) -> bool {
        Self::find_led_path(lock_type)
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|c| c.trim() == "1")
            .unwrap_or(false)
    }
}