#![allow(dead_code)]
use gpui::*;
use std::process::Command;

pub struct LockStateService {
    is_locked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockStateChanged {
    pub is_locked: bool,
    pub lock_type: LockType,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockType {
    CapsLock,
    Screen,
}

pub struct LockMonitor {
    is_locked: bool,
}

impl EventEmitter<LockStateChanged> for LockMonitor {}

impl LockMonitor {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let is_locked = Self::check_lock_state();
        let model = cx.new(|_| Self { is_locked });

        let weak_model = model.downgrade();
        cx.spawn(move |cx: &mut AsyncApp| {
            // Added move
            let mut cx = cx.clone();
            let mut last_state = is_locked;
            async move {
                loop {
                    let current_state = Self::check_lock_state();
                    if current_state != last_state {
                        last_state = current_state;
                        let _ = weak_model.update(&mut cx, |this, cx| {
                            this.is_locked = current_state;
                            cx.emit(LockStateChanged {
                                is_locked: current_state,
                                lock_type: LockType::Screen,
                                enabled: current_state,
                            });
                        });
                    }
                    cx.background_executor()
                        .timer(std::time::Duration::from_secs(1))
                        .await;
                }
            }
        })
        .detach();

        model
    }

    fn check_lock_state() -> bool {
        let output = Command::new("pgrep").arg("-x").arg("hyprlock").output();

        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
}

impl Global for LockStateService {}

impl LockStateService {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let is_locked = Self::check_lock_state();
        let model = cx.new(|_| Self { is_locked });
        cx.set_global(LockStateService { is_locked });

        let weak_model = model.downgrade();
        cx.spawn(move |cx: &mut AsyncApp| {
            // Added move
            let mut cx = cx.clone();
            let mut last_state = is_locked;
            async move {
                loop {
                    let current_state = Self::check_lock_state();
                    if current_state != last_state {
                        last_state = current_state;
                        let _ = weak_model.update(&mut cx, |this, cx| {
                            this.is_locked = current_state;
                            cx.update_global::<LockStateService, _>(|service, _| {
                                service.is_locked = current_state;
                            });
                        });
                    }
                    cx.background_executor()
                        .timer(std::time::Duration::from_secs(1))
                        .await;
                }
            }
        })
        .detach();

        model
    }

    fn check_lock_state() -> bool {
        LockMonitor::check_lock_state()
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }
}
