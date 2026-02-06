use crate::assets::Icon;
use crate::theme::ActiveTheme;
use crate::widgets::notifications::types::Notification;
use crate::widgets::notifications::widget::time_formatter::format_time_ago;
use gpui::prelude::*;
use gpui::*;

pub fn render_notification_item(notif: &Notification, cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();

    let urgency_class = match notif.urgency {
        2 => theme.red,
        1 => theme.bg,
        _ => theme.hover,
    };

    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .bg(urgency_class)
        .rounded(px(12.0))
        .border_1()
        .border_color(theme.accent_alt.opacity(0.25))
        .shadow_lg()
        .child(render_header(notif, &theme))
        .child(render_summary(notif, &theme))
        .when(!notif.body.as_ref().is_empty(), |this| {
            this.child(render_body(notif, &theme))
        })
}

fn render_header(notif: &Notification, theme: &crate::theme::Theme) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .gap_2()
                .items_center()
                .child(render_icon(notif, theme))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text)
                        .child(notif.app_name.clone()),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(format_time_ago(notif.timestamp)),
        )
}

fn render_icon(notif: &Notification, theme: &crate::theme::Theme) -> AnyElement {
    if !notif.app_icon.as_ref().is_empty() {
        Icon::new(notif.app_icon.to_string())
            .size(px(20.0))
            .color(theme.text)
            .preserve_colors(true)
            .into_any_element()
    } else {
        div().size_4().into_any_element()
    }
}

fn render_summary(notif: &Notification, theme: &crate::theme::Theme) -> impl IntoElement {
    div()
        .text_base()
        .font_weight(FontWeight::BOLD)
        .text_color(theme.text)
        .child(notif.summary.clone())
}

fn render_body(notif: &Notification, theme: &crate::theme::Theme) -> impl IntoElement {
    div()
        .text_sm()
        .text_color(theme.text_bright)
        .child(notif.body.clone())
}
