use gpui::SharedString;

pub const MAX_NOTIFICATIONS: usize = 10;
pub const NOTIFICATION_TIMEOUT_SECS: u64 = 5;
pub const HISTORY_CAPACITY: usize = 50;

#[derive(Clone, Debug)]
pub struct Notification {
    pub app_name: SharedString,
    pub summary: SharedString,
    pub body: SharedString,
    pub urgency: u8,
    pub timestamp: u64,
    pub actions: Vec<String>,
    pub app_icon: SharedString,
}

#[derive(Clone)]
pub struct NotificationAdded {
    pub notification: Notification,
}

#[derive(Clone)]
pub struct NotificationsEmpty;

#[derive(Clone)]
pub struct NotificationsStateChanged {
    pub has_notifications: bool,
}
