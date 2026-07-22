use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Entity, EventEmitter, Global};
use std::fs;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockStateChanged {
    pub capslock_enabled: bool,
}

pub struct LockMonitor {
    pub capslock_enabled: bool,
}

impl EventEmitter<LockStateChanged> for LockMonitor {}

struct GlobalLockMonitor(Entity<LockMonitor>);
impl Global for GlobalLockMonitor {}

impl LockMonitor {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalLockMonitor>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let initial_state = Self::read_capslock_state();
        let service = cx.new(|_| Self {
            capslock_enabled: initial_state,
        });
        cx.set_global(GlobalLockMonitor(service.clone()));

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<bool>();

        gpui_tokio::Tokio::spawn(cx, async move {
            let mut last_state = Self::read_capslock_state();

            loop {
                tokio::time::sleep(Duration::from_millis(300)).await;
                let current_state = Self::read_capslock_state();
                if current_state != last_state {
                    last_state = current_state;
                    if tx.unbounded_send(current_state).is_err() {
                        break;
                    }
                }
            }
        })
        .detach();

        let weak = service.downgrade();
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(caps_on) = rx.next().await {
                    let _ = weak.update(&mut cx, |this, cx| {
                        if this.capslock_enabled != caps_on {
                            this.capslock_enabled = caps_on;
                            cx.emit(LockStateChanged {
                                capslock_enabled: caps_on,
                            });
                            cx.notify();
                        }
                    });
                }
            }
        })
        .detach();

        service
    }

    pub fn read_capslock_state() -> bool {
        if let Ok(entries) = fs::read_dir("/sys/class/leds") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.contains("capslock") {
                        let brightness_path = path.join("brightness");
                        if let Ok(content) = fs::read_to_string(&brightness_path) {
                            if let Ok(brightness) = content.trim().parse::<u8>() {
                                return brightness > 0;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}
