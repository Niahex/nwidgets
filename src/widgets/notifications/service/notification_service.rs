use crate::widgets::notifications::service::dbus_server::run_dbus_server;
use crate::widgets::notifications::service::state::STATE;
use crate::widgets::notifications::types::{Notification, NotificationAdded, NotificationsEmpty};
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct NotificationService {
    pub notifications: Arc<RwLock<Vec<Notification>>>,
}

impl EventEmitter<NotificationAdded> for NotificationService {}
impl EventEmitter<NotificationsEmpty> for NotificationService {}

impl NotificationService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self::start_dbus_server(cx);

        let notifications = Arc::new(RwLock::new(Vec::new()));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        {
            let mut state = STATE.lock();
            state.sender = Some(tx);
        }

        let notifications_clone = Arc::clone(&notifications);

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(notification) = rx.recv().await {
                    notifications_clone.write().push(notification.clone());
                    let _ = this.update(&mut cx, |_, cx| {
                        cx.emit(NotificationAdded { notification });
                        cx.notify();
                    });
                }
            }
        })
        .detach();

        Self { notifications }
    }

    fn start_dbus_server(cx: &mut Context<Self>) {
        static INIT: std::sync::Once = std::sync::Once::new();

        INIT.call_once(|| {
            log::info!("Starting notification D-Bus server");
            let state_ref = Arc::clone(&STATE);

            gpui_tokio::Tokio::spawn(cx, async move {
                if let Err(e) = run_dbus_server(state_ref).await {
                    log::error!("Notification D-Bus error: {e}");
                }
            })
            .detach();
        });
    }

    pub fn get_all(&self) -> Vec<Notification> {
        self.notifications.read().clone()
    }

    pub fn clear(&self) {
        self.notifications.write().clear();
    }
}

struct GlobalNotificationService(Entity<NotificationService>);
impl Global for GlobalNotificationService {}

impl NotificationService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalNotificationService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalNotificationService(service.clone()));
        service
    }
}
