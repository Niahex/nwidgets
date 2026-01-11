use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Entity, EventEmitter, Global};
use zbus::{proxy, Connection, Result};

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

// --- DBus Interfaces ---

#[proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1/session/auto"
)]
trait Session {
    #[zbus(property)]
    fn locked_hint(&self) -> Result<bool>;

    #[zbus(signal)]
    fn lock(&self) -> Result<()>;

    #[zbus(signal)]
    fn unlock(&self) -> Result<()>;
}

// --- Service Implementation ---

impl LockMonitor {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let is_locked = false;
        let service = cx.new(|_| Self { is_locked });

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<bool>();

        // 1. Worker Task (Tokio)
        gpui_tokio::Tokio::spawn(cx, async move { Self::lock_monitor_worker(tx).await }).detach();

        // 2. UI Task (GPUI)
        let weak_service = service.downgrade();
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let weak_service = weak_service.clone();
            async move {
                while let Some(locked) = rx.next().await {
                    let _ = weak_service.update(&mut cx, |this, cx| {
                        if this.is_locked != locked {
                            this.is_locked = locked;

                            if cx.has_global::<LockStateService>() {
                                cx.update_global::<LockStateService, _>(|service, _| {
                                    service.is_locked = locked;
                                });
                            }

                            cx.emit(LockStateChanged {
                                is_locked: locked,
                                lock_type: LockType::Screen,
                                enabled: locked,
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
        self.is_locked
    }

    async fn lock_monitor_worker(tx: futures::channel::mpsc::UnboundedSender<bool>) {
        let conn = match Connection::system().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[LockMonitor] Failed to connect to system bus: {e}");
                return;
            }
        };

        let session = match SessionProxy::new(&conn).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[LockMonitor] Failed to create Session proxy: {e}");
                return;
            }
        };

        if let Ok(locked) = session.locked_hint().await {
            let _ = tx.unbounded_send(locked);
        }

        let mut lock_stream = match session.receive_lock().await {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("[LockMonitor] Failed to receive Lock signal: {e}");
                None
            }
        };
        let mut unlock_stream = match session.receive_unlock().await {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("[LockMonitor] Failed to receive Unlock signal: {e}");
                None
            }
        };

        let mut locked_hint_stream = session.receive_locked_hint_changed().await;

        loop {
            tokio::select! {
                Some(_) = async {
                    if let Some(s) = &mut lock_stream { s.next().await } else { std::future::pending().await }
                } => {
                    let _ = tx.unbounded_send(true);
                }
                Some(_) = async {
                    if let Some(s) = &mut unlock_stream { s.next().await } else { std::future::pending().await }
                } => {
                    let _ = tx.unbounded_send(false);
                }
                Some(_) = locked_hint_stream.next() => {
                    if let Ok(locked) = session.locked_hint().await {
                        let _ = tx.unbounded_send(locked);
                    }
                }
            }
        }
    }
}

pub struct LockStateService {
    is_locked: bool,
}

impl Global for LockStateService {}

impl LockStateService {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let is_locked = false;
        let service = cx.new(|_| Self { is_locked });
        cx.set_global(LockStateService { is_locked });
        LockMonitor::init(cx);
        service
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }
}
