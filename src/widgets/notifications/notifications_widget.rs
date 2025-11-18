use super::{Notification, NotificationService};
use gpui::{div, prelude::*, rgb, Context, Window};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Nord Dark palette
const NORD2: u32 = 0x434c5e;
const NORD3: u32 = 0x4c566a;
const NORD4: u32 = 0xd8dee9;
const NORD8: u32 = 0x88c0d0;
const NORD11: u32 = 0xbf616a;
const NORD13: u32 = 0xebcb8b;

pub struct NotificationsWidget {
    notifications: Vec<Notification>,
}

impl NotificationsWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (service, mut receiver) = NotificationService::new();
        service.start_dbus_server();

        let widget = Self {
            notifications: Vec::new(),
        };

        // Réception des notifications
        cx.spawn(async move |this, cx| {
            println!("[NOTIFICATIONS] 📢 Notification receiver task started");
            while let Some(notification) = receiver.recv().await {
                println!(
                    "[NOTIFICATIONS] 📢 Received notification from channel: {} - {}",
                    notification.summary, notification.body
                );
                let result = this.update(cx, |widget, cx| {
                    println!(
                        "[NOTIFICATIONS] 📢 Adding notification to widget (current count: {})",
                        widget.notifications.len()
                    );
                    widget.notifications.push(notification.clone());
                    widget
                        .notifications
                        .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    widget.notifications.truncate(10);
                    println!(
                        "[NOTIFICATIONS] 📢 Now have {} notifications, calling cx.notify()",
                        widget.notifications.len()
                    );
                    cx.notify();
                });
                if let Err(e) = result {
                    println!(
                        "[NOTIFICATIONS] ❌ Error updating widget with notification: {:?}",
                        e
                    );
                }
            }
            println!("[NOTIFICATIONS] ⚠️  Notification receiver channel closed!");
        })
        .detach();

        // Timer pour nettoyer les notifications expirées
        cx.spawn(async move |this, cx| {
            println!("[NOTIFICATIONS] 📢 Notification cleanup task started");
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let result = this.update(cx, |widget, cx| {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let old_count = widget.notifications.len();
                    widget.notifications.retain(|n| now - n.timestamp < 5);
                    if widget.notifications.len() != old_count {
                        println!(
                            "[NOTIFICATIONS] 📢 Cleaned up notifications: {} -> {}",
                            old_count,
                            widget.notifications.len()
                        );
                        cx.notify();
                    }
                });
                if let Err(e) = result {
                    println!("[NOTIFICATIONS] ❌ Error cleaning notifications: {:?}", e);
                }
            }
        })
        .detach();

        widget
    }
}

impl Render for NotificationsWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut container = div().size_full().flex().flex_col().gap_3().p_4();

        for notification in &self.notifications {
            let border_color = match notification.urgency {
                2 => NORD11, // Critical - rouge
                1 => NORD13, // Normal - jaune
                _ => NORD8,  // Low - bleu
            };

            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - notification.timestamp;

            let time_str = if elapsed < 60 {
                "now".to_string()
            } else if elapsed < 3600 {
                format!("{}m ago", elapsed / 60)
            } else {
                format!("{}h ago", elapsed / 3600)
            };

            let mut notif_div = div()
                .w_full()
                .bg(rgb(NORD2))
                .rounded_lg()
                .p_4()
                .border_l_4()
                .border_color(rgb(border_color))
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_color(rgb(NORD4))
                                .text_sm()
                                .font_weight(gpui::FontWeight::BOLD)
                                .child(notification.summary.clone()),
                        )
                        .child(div().text_color(rgb(NORD3)).text_xs().child(time_str)),
                )
                .child(
                    div()
                        .text_color(rgb(NORD4))
                        .text_xs()
                        .child(notification.body.clone()),
                );

            if !notification.app_name.is_empty() {
                notif_div = notif_div.child(
                    div()
                        .text_color(rgb(NORD3))
                        .text_xs()
                        .child(format!("from {}", notification.app_name)),
                );
            }

            container = container.child(notif_div);
        }

        container
    }
}
