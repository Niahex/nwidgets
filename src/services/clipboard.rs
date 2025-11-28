use crate::utils::subscription::ServiceSubscription;
use once_cell::sync::Lazy;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

static SUBSCRIBERS: Lazy<Arc<Mutex<Vec<Sender<()>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

static MONITOR_STARTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub struct ClipboardService;

impl ClipboardService {
    pub fn get_clipboard_content() -> Option<String> {
        Command::new("wl-paste")
            .arg("-n")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
    }

    pub fn subscribe_clipboard<F>(callback: F)
    where
        F: Fn() + 'static,
    {
        let mut started = MONITOR_STARTED.lock().unwrap();
        if !*started {
            *started = true;
            drop(started);
            Self::start_monitoring();
        }

        let (tx, rx) = std::sync::mpsc::channel();
        SUBSCRIBERS.lock().unwrap().push(tx);

        ServiceSubscription::subscribe(rx, move |_| callback());
    }

    fn start_monitoring() {
        let subscribers = Arc::clone(&SUBSCRIBERS);

        thread::spawn(move || {
            let mut last_content = Self::get_clipboard_content();

            loop {
                thread::sleep(Duration::from_millis(100));
                let current_content = Self::get_clipboard_content();

                if current_content != last_content && current_content.is_some() {
                    last_content = current_content;

                    let mut subs = subscribers.lock().unwrap();
                    subs.retain(|tx| tx.send(()).is_ok());
                }
            }
        });
    }
}
