use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

impl super::ControlCenterWidget {
    pub(super) fn render_notifications_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let notifications = self.notifications.read(cx).get_all();
        let theme = cx.theme();
        let notif_service = self.notifications.clone();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_size(px(16.)).font_weight(FontWeight::BOLD).text_color(theme.text).child("Notifications"))
                    .when(!notifications.is_empty(), |this| {
                        this.child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(theme.surface)
                                .text_xs()
                                .text_color(theme.text_muted)
                                .cursor_pointer()
                                .hover(|style| style.bg(theme.overlay))
                                .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |_, _, _, cx| {
                                    notif_service.read(cx).clear();
                                    cx.notify();
                                }))
                                .child("Clear"),
                        )
                    }),
            )
            .children(notifications.iter().take(5).map(|n| {
                div()
                    .bg(theme.surface)
                    .rounded_md()
                    .p_2()
                    .mb_1()
                    .child(div().font_weight(FontWeight::BOLD).text_color(theme.text).child(n.summary.clone()))
                    .when(!n.body.is_empty(), |this| {
                        this.child(div().text_size(px(12.0)).text_color(theme.text_muted).child(n.body.clone()))
                    })
            }))
            .when(notifications.is_empty(), |this| {
                this.child(div().text_xs().text_color(theme.text_muted).child("No notifications"))
            })
    }
}
