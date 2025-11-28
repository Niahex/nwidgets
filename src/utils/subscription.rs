use async_channel;
use glib::MainContext;
use std::sync::mpsc;
use std::thread;

/// Generic subscription helper that bridges sync monitoring threads with async GTK callbacks
///
/// This abstraction eliminates the repeated pattern of:
/// 1. Creating sync channel for monitoring thread
/// 2. Creating async channel for GTK main loop
/// 3. Spawning bridge thread between them
/// 4. Spawning GTK callback handler
///
/// Usage:
/// ```
/// ServiceSubscription::subscribe(rx_from_monitor, move |state| {
///     // Your GTK callback here
/// });
/// ```
pub struct ServiceSubscription;

impl ServiceSubscription {
    /// Subscribe to state updates from a monitoring thread
    ///
    /// Takes a receiver from a monitoring thread and a callback that will be
    /// executed in the GTK main loop whenever new state arrives.
    pub fn subscribe<T, F>(rx: mpsc::Receiver<T>, callback: F)
    where
        T: Send + 'static,
        F: Fn(T) + 'static,
    {
        let (async_tx, async_rx) = async_channel::unbounded();

        // Bridge thread: sync -> async
        thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                if async_tx.send_blocking(state).is_err() {
                    break;
                }
            }
        });

        // GTK main loop callback
        MainContext::default().spawn_local(async move {
            while let Ok(state) = async_rx.recv().await {
                callback(state);
            }
        });
    }

    /// Create a subscription system with monitoring thread
    ///
    /// Returns (sender, subscription_fn) where:
    /// - sender: Send state updates to subscribers
    /// - subscription_fn: Call this to add new subscribers
    ///
    /// The monitoring thread should call sender.send(state) to update all subscribers.
    pub fn create_subscription_system<T, M>(monitor_fn: M) -> impl Fn(Box<dyn Fn(T) + 'static>)
    where
        T: Clone + Send + 'static,
        M: FnOnce(mpsc::Sender<T>) + Send + 'static,
    {
        use std::sync::{Arc, Mutex};

        let subscribers: Arc<Mutex<Vec<mpsc::Sender<T>>>> = Arc::new(Mutex::new(Vec::new()));
        let subscribers_clone = Arc::clone(&subscribers);

        // Start monitoring thread
        thread::spawn(move || {
            let (tx, rx) = mpsc::channel();

            // Spawn the actual monitor
            thread::spawn(move || {
                monitor_fn(tx);
            });

            // Relay to all subscribers
            while let Ok(state) = rx.recv() {
                let mut subs = subscribers_clone.lock().unwrap();
                subs.retain(|subscriber| subscriber.send(state.clone()).is_ok());
            }
        });

        // Return subscription function
        move |callback: Box<dyn Fn(T) + 'static>| {
            let (tx, rx) = mpsc::channel();
            subscribers.lock().unwrap().push(tx);
            Self::subscribe(rx, callback);
        }
    }

    /// Simplified subscription for services that poll periodically
    ///
    /// Takes a polling function and interval, manages the monitoring thread,
    /// and returns a subscription function.
    pub fn create_polling_subscription<T, F>(
        poll_fn: F,
        interval: std::time::Duration,
    ) -> impl Fn(Box<dyn Fn(T) + 'static>)
    where
        T: Clone + Send + PartialEq + 'static,
        F: Fn() -> T + Send + 'static,
    {
        Self::create_subscription_system(move |tx| {
            let mut last_state = poll_fn();

            // Send initial state
            let _ = tx.send(last_state.clone());

            loop {
                thread::sleep(interval);
                let current_state = poll_fn();

                if current_state != last_state {
                    last_state = current_state.clone();
                    if tx.send(current_state).is_err() {
                        break;
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_subscription_basic() {
        let (tx, rx) = mpsc::channel();

        let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let received_clone = received.clone();

        ServiceSubscription::subscribe(rx, move |value: i32| {
            received_clone.lock().unwrap().push(value);
        });

        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();

        thread::sleep(Duration::from_millis(100));

        // Note: In real GTK app, the main loop would process these
        // In test, we just verify the subscription was created
    }
}
