use gpui::{Context, Window, div, prelude::*, rgb, px, AnyElement};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::modules::{Notification, NotificationService};

// Nord Dark palette
const NORD0: u32 = 0x2e3440;
const NORD1: u32 = 0x3b4252;
const NORD2: u32 = 0x434c5e;
const NORD3: u32 = 0x4c566a;
const NORD4: u32 = 0xd8dee9;
const NORD8: u32 = 0x88c0d0;
const NORD9: u32 = 0x81a1c1;
const NORD10: u32 = 0x5e81ac;
const NORD11: u32 = 0xbf616a;
const NORD13: u32 = 0xebcb8b;
const NORD14: u32 = 0xa3be8c;

pub enum ShellMode {
    Background,
    Panel,
    Notifications,
}

pub struct Shell {
    mode: ShellMode,
    notification_service: Option<NotificationService>,
    notifications: Vec<Notification>,
}

impl Shell {
    pub fn new_background(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::Background,
            notification_service: None,
            notifications: Vec::new(),
        }
    }

    pub fn new_panel(_cx: &mut Context<Self>) -> Self {
        Self {
            mode: ShellMode::Panel,
            notification_service: None,
            notifications: Vec::new(),
        }
    }

    pub fn new_notifications(cx: &mut Context<Self>) -> Self {
        let (service, mut receiver) = NotificationService::new();
        
        let shell = Self {
            mode: ShellMode::Notifications,
            notification_service: Some(service),
            notifications: Vec::new(),
        };

        cx.spawn(async move |this, cx| {
            while let Some(notification) = receiver.recv().await {
                let _ = this.update(cx, |shell, cx| {
                    shell.notifications.push(notification);
                    shell.notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    shell.notifications.truncate(10);
                    cx.notify();
                });
            }
        }).detach();

        shell
    }

    fn render_background(&self) -> AnyElement {
        div()
            .size_full()
            .bg(rgb(NORD0))
            .child(
                div()
                    .size_full()
                    .bg(rgb(NORD1))
            )
            .child(
                div()
                    .absolute()
                    .bottom(px(20.))
                    .right(px(20.))
                    .p_4()
                    .bg(rgb(NORD2))
                    .rounded_md()
                    .text_color(rgb(NORD4))
                    .child("12:34")
            )
            .into_any_element()
    }

    fn render_panel(&self) -> AnyElement {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hours = (now / 3600) % 24;
        let minutes = (now / 60) % 60;

        div()
            .size_full()
            .bg(rgb(NORD1))
            .border_r_1()
            .border_color(rgb(NORD3))
            .flex()
            .flex_col()
            .justify_between()
            .py_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w_10()
                            .h_10()
                            .bg(rgb(NORD8))
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD0))
                            .text_sm()
                            .child("📱")
                    )
                    .child(
                        div()
                            .w_10()
                            .h_10()
                            .bg(rgb(NORD9))
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD0))
                            .text_sm()
                            .child("🚀")
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w_8()
                            .h_8()
                            .bg(rgb(NORD10))
                            .rounded_sm()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD4))
                            .text_xs()
                            .child("1")
                    )
                    .child(
                        div()
                            .w_8()
                            .h_8()
                            .bg(rgb(NORD2))
                            .rounded_sm()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD4))
                            .text_xs()
                            .child("2")
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w_10()
                            .h_8()
                            .bg(rgb(NORD14))
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD0))
                            .text_sm()
                            .child("🔊")
                    )
                    .child(
                        div()
                            .w_10()
                            .h_8()
                            .bg(rgb(NORD13))
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD0))
                            .text_sm()
                            .child("🔋")
                    )
                    .child(
                        div()
                            .w_10()
                            .h_12()
                            .bg(rgb(NORD3))
                            .rounded_md()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(NORD4))
                            .text_xs()
                            .child(format!("{:02}", hours))
                            .child(format!("{:02}", minutes))
                    )
            )
            .into_any_element()
    }

    fn render_notifications(&self) -> AnyElement {
        let mut container = div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4();

        for notification in &self.notifications {
            let border_color = match notification.urgency {
                2 => NORD11, // Critical - rouge
                1 => NORD13, // Normal - jaune
                _ => NORD8,  // Low - bleu
            };

            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() - notification.timestamp;

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
                                .child(notification.summary.clone())
                        )
                        .child(
                            div()
                                .text_color(rgb(NORD3))
                                .text_xs()
                                .child(time_str)
                        )
                )
                .child(
                    div()
                        .text_color(rgb(NORD4))
                        .text_xs()
                        .child(notification.body.clone())
                );

            if !notification.app_name.is_empty() {
                notif_div = notif_div.child(
                    div()
                        .text_color(rgb(NORD3))
                        .text_xs()
                        .child(format!("from {}", notification.app_name))
                );
            }

            container = container.child(notif_div);
        }

        if self.notifications.is_empty() {
            container = container.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h_full()
                    .text_color(rgb(NORD3))
                    .text_sm()
                    .child("No notifications")
            );
        }

        container.into_any_element()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self.mode {
            ShellMode::Background => self.render_background(),
            ShellMode::Panel => self.render_panel(),
            ShellMode::Notifications => self.render_notifications(),
        }
    }
}
