use gpui::*;
use crate::services::notifications::NotificationService;
use std::sync::Arc;

pub struct NotificationsWidget {
    service: Arc<NotificationService>,
}

impl NotificationsWidget {
    pub fn new(service: Arc<NotificationService>) -> Self {
        Self { service }
    }
}

impl Render for NotificationsWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let notifications = self.service.get_all();
        
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .bg(rgb(0x2e2e2e))
            .rounded_lg()
            .children(notifications.iter().map(|notif| {
                div()
                    .flex()
                    .flex_col()
                    .p_3()
                    .bg(rgb(0x3e3e3e))
                    .rounded_md()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .child(notif.summary.clone())
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xaaaaaa))
                            .child(notif.body.clone())
                    )
            }))
    }
}
