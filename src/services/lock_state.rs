use std::fs;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use glib::MainContext;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    CapsLock,
    NumLock,
}

impl LockType {
    fn sysfs_path(&self) -> &str {
        match self {
            LockType::CapsLock => "/sys/class/leds/input0::capslock/brightness",
            LockType::NumLock => "/sys/class/leds/input0::numlock/brightness",
        }
    }

    fn name(&self) -> &str {
        match self {
            LockType::CapsLock => "CapsLock",
            LockType::NumLock => "NumLock",
        }
    }
}

struct LockStateMonitor {
    lock_type: LockType,
    subscribers: Arc<Mutex<Vec<Sender<bool>>>>,
    monitor_started: Mutex<bool>,
}

impl LockStateMonitor {
    fn new(lock_type: LockType) -> Self {
        Self {
            lock_type,
            subscribers: Arc::new(Mutex::new(Vec::new())),
            monitor_started: Mutex::new(false),
        }
    }

    fn get_state(&self) -> bool {
        if let Ok(content) = fs::read_to_string(self.lock_type.sysfs_path()) {
            let trimmed = content.trim();
            // Support both "1" string and numeric comparison
            return trimmed == "1" || trimmed.parse::<u8>().unwrap_or(0) > 0;
        }
        false
    }

    fn subscribe<F>(&self, callback: F)
    where
        F: Fn(bool) + 'static,
    {
        let mut started = self.monitor_started.lock().unwrap();
        if !*started {
            *started = true;
            drop(started);
            self.start_monitoring();
        }

        let (tx, rx) = mpsc::channel();
        let (async_tx, async_rx) = async_channel::unbounded();

        self.subscribers.lock().unwrap().push(tx);

        thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                if async_tx.send_blocking(state).is_err() {
                    break;
                }
            }
        });

        MainContext::default().spawn_local(async move {
            while let Ok(state) = async_rx.recv().await {
                callback(state);
            }
        });
    }

    fn start_monitoring(&self) {
        let subscribers = Arc::clone(&self.subscribers);
        let lock_type = self.lock_type;

        thread::spawn(move || {
            let monitor = LockStateMonitor::new(lock_type);
            let mut last_state = monitor.get_state();

            loop {
                thread::sleep(Duration::from_millis(100));
                let current_state = monitor.get_state();

                if current_state != last_state {
                    last_state = current_state;

                    let mut subs = subscribers.lock().unwrap();
                    subs.retain(|tx| tx.send(current_state).is_ok());
                }
            }
        });
    }
}

static CAPSLOCK_MONITOR: Lazy<LockStateMonitor> =
    Lazy::new(|| LockStateMonitor::new(LockType::CapsLock));

static NUMLOCK_MONITOR: Lazy<LockStateMonitor> =
    Lazy::new(|| LockStateMonitor::new(LockType::NumLock));

pub struct CapsLockService;

impl CapsLockService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    pub fn get_caps_lock_state() -> bool {
        CAPSLOCK_MONITOR.get_state()
    }

    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        Self::get_caps_lock_state()
    }

    pub fn subscribe_capslock<F>(callback: F)
    where
        F: Fn(bool) + 'static,
    {
        CAPSLOCK_MONITOR.subscribe(callback);
    }
}

pub struct NumLockService;

impl NumLockService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    pub fn get_num_lock_state() -> bool {
        NUMLOCK_MONITOR.get_state()
    }

    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        Self::get_num_lock_state()
    }

    pub fn subscribe_numlock<F>(callback: F)
    where
        F: Fn(bool) + 'static,
    {
        NUMLOCK_MONITOR.subscribe(callback);
    }
}
