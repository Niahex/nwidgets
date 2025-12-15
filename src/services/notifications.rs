use gpui::prelude::*;
use gpui::{App, Context, Entity, EventEmitter, Global};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: Option<String>,
}

#[derive(Clone)]
pub struct NotificationAdded {
    pub notification: Notification,
}

pub struct NotificationService {
    notifications: Arc<RwLock<Vec<Notification>>>,
}

impl EventEmitter<NotificationAdded> for NotificationService {}

impl NotificationService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            notifications: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add(&self, notification: Notification, cx: &mut Context<Self>) {
        self.notifications.write().push(notification.clone());
        cx.emit(NotificationAdded { notification });
        cx.notify();
    }

    pub fn get_all(&self) -> Vec<Notification> {
        self.notifications.read().clone()
    }

    pub fn clear(&self, cx: &mut Context<Self>) {
        self.notifications.write().clear();
        cx.notify();
    }
}

// Global accessor
struct GlobalNotificationService(Entity<NotificationService>);
impl Global for GlobalNotificationService {}

impl NotificationService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalNotificationService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|cx| Self::new(cx));
        cx.set_global(GlobalNotificationService(service.clone()));
        service
    }
}
