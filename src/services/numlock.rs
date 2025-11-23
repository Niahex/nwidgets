use std::fs;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use glib::MainContext;
use once_cell::sync::Lazy;

static SUBSCRIBERS: Lazy<Arc<Mutex<Vec<Sender<bool>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

static MONITOR_STARTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub struct NumLockService;

impl NumLockService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_num_lock_state() -> bool {
        if let Ok(content) = fs::read_to_string("/sys/class/leds/input0::numlock/brightness") {
            return content.trim() == "1";
        }
        false
    }

    pub fn is_enabled(&self) -> bool {
        Self::get_num_lock_state()
    }

    pub fn subscribe_numlock<F>(callback: F)
    where
        F: Fn(bool) + 'static,
    {
        let mut started = MONITOR_STARTED.lock().unwrap();
        if !*started {
            *started = true;
            drop(started);
            Self::start_monitoring();
        }

        let (async_tx, async_rx) = async_channel::unbounded();
        let (tx, rx) = mpsc::channel();

        SUBSCRIBERS.lock().unwrap().push(tx);

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

    fn start_monitoring() {
        let subscribers = Arc::clone(&SUBSCRIBERS);

        thread::spawn(move || {
            let mut last_state = Self::get_num_lock_state();

            loop {
                thread::sleep(Duration::from_millis(100));
                let current_state = Self::get_num_lock_state();

                if current_state != last_state {
                    last_state = current_state;

                    let mut subs = subscribers.lock().unwrap();
                    subs.retain(|tx| tx.send(current_state).is_ok());
                }
            }
        });
    }
}
