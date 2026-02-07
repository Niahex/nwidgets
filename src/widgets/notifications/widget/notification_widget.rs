use crate::widgets::notifications::service::NotificationService;
use crate::widgets::notifications::types::{
    Notification, NotificationAdded, NotificationsStateChanged, MAX_NOTIFICATIONS,
    NOTIFICATION_TIMEOUT_SECS,
};
use crate::widgets::notifications::widget::notification_item::render_notification_item;
use gpui::prelude::*;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct NotificationsWidget {
    #[allow(dead_code)]
    service: Entity<NotificationService>,
    notifications: Arc<RwLock<Vec<Notification>>>,
}

impl EventEmitter<NotificationsStateChanged> for NotificationsWidget {}

impl NotificationsWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = NotificationService::global(cx);
        let notifications = Arc::new(RwLock::new(Vec::with_capacity(20)));

        Self::subscribe_to_notifications(&service, Arc::clone(&notifications), cx);
        Self::start_cleanup_timer(Arc::clone(&notifications), cx);

        Self {
            service,
            notifications,
        }
    }

    fn subscribe_to_notifications(
        service: &Entity<NotificationService>,
        notifications: Arc<RwLock<Vec<Notification>>>,
        cx: &mut Context<Self>,
    ) {
        cx.subscribe(
            service,
            move |_this, _service, event: &NotificationAdded, cx| {
                let mut notifs = notifications.write();

                notifs.insert(0, event.notification.clone());
                notifs.truncate(MAX_NOTIFICATIONS);

                drop(notifs);
                cx.emit(NotificationsStateChanged {
                    has_notifications: true,
                });
                cx.notify();
            },
        )
        .detach();
    }

    fn start_cleanup_timer(
        notifications: Arc<RwLock<Vec<Notification>>>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let mut notifs = notifications.write();
            let old_count = notifs.len();
            notifs.retain(|n| now - n.timestamp < NOTIFICATION_TIMEOUT_SECS);

            if notifs.len() != old_count {
                let is_empty = notifs.is_empty();
                drop(notifs);
                let _ = this.update(cx, |_, cx| {
                    if is_empty {
                        cx.emit(NotificationsStateChanged {
                            has_notifications: false,
                        });
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }
}

impl Render for NotificationsWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notifs = self.notifications.read().clone();

        if notifs.is_empty() {
            return div().into_any_element();
        }

        div()
            .flex()
            .flex_col()
            .gap_2()
            .w(px(380.0))
            .children(notifs.iter().map(|notif| render_notification_item(notif, cx)))
            .with_animation(
                "notifications-fade-in",
                Animation::new(Duration::from_millis(150)),
                |this, delta| this.opacity(delta),
            )
            .into_any_element()
    }
}
