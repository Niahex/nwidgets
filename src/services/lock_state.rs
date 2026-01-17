use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Entity, EventEmitter};
use std::fs;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockStateChanged {
    pub is_locked: bool,
    pub lock_type: LockType,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockType {
    CapsLock,
}

pub struct LockMonitor {
    capslock_state: bool,
}

impl EventEmitter<LockStateChanged> for LockMonitor {}

impl LockMonitor {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let capslock_state = Self::read_capslock_state();
        let service = cx.new(|_| Self { capslock_state });

        let (caps_tx, mut caps_rx) = futures::channel::mpsc::unbounded::<bool>();

        // Worker Task (Tokio) - CapsLock
        gpui_tokio::Tokio::spawn(cx, async move { Self::capslock_monitor_worker(caps_tx).await }).detach();

        // UI Task (GPUI) - CapsLock
        let weak_service = service.downgrade();
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let weak_service = weak_service.clone();
            async move {
                while let Some(caps_on) = caps_rx.next().await {
                    let _ = weak_service.update(&mut cx, |this, cx| {
                        if this.capslock_state != caps_on {
                            this.capslock_state = caps_on;
                            cx.emit(LockStateChanged {
                                is_locked: false,
                                lock_type: LockType::CapsLock,
                                enabled: caps_on,
                            });
                        }
                    });
                }
            }
        })
        .detach();

        service
    }

    pub fn is_locked(&self) -> bool {
        self.capslock_state
    }

    fn read_capslock_state() -> bool {
        // Lire l'état CapsLock via /sys/class/leds/input*::capslock/brightness
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

    async fn capslock_monitor_worker(tx: futures::channel::mpsc::UnboundedSender<bool>) {
        let mut last_state = Self::read_capslock_state();
        let _ = tx.unbounded_send(last_state);

        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let current_state = Self::read_capslock_state();
            if current_state != last_state {
                last_state = current_state;
                let _ = tx.unbounded_send(current_state);
            }
        }
    }
}
