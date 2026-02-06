use crate::widgets::notifications::types::{Notification, HISTORY_CAPACITY};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct NotificationState {
    pub sender: Option<tokio::sync::mpsc::UnboundedSender<Notification>>,
    pub history: VecDeque<Notification>,
}

impl NotificationState {
    pub fn new() -> Self {
        Self {
            sender: None,
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
        }
    }
}

pub static STATE: once_cell::sync::Lazy<Arc<Mutex<NotificationState>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(NotificationState::new())));
