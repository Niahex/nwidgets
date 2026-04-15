use crate::theme::Theme;
use chrono::{Duration, Local, NaiveDate};
use gpui::prelude::*;
use gpui::*;

pub struct DateItem {
    pub date: NaiveDate,
    pub is_selected: bool,
    pub is_today: bool,
}

pub fn build_date_items(selected_date: NaiveDate) -> Vec<DateItem> {
    let today = Local::now().date_naive();
    (-3..=3)
        .map(|i| {
            let date = selected_date + Duration::days(i);
            DateItem {
                date,
                is_selected: date == selected_date,
                is_today: date == today,
            }
        })
        .collect()
}

pub fn render_date_item(item: &DateItem, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .px_3()
        .py_2()
        .rounded_md()
        .cursor_pointer()
        .bg(if item.is_selected {
            theme.accent.opacity(0.2)
        } else {
            theme.surface
        })
        .border_1()
        .border_color(if item.is_selected {
            theme.accent
        } else if item.is_today {
            theme.accent.opacity(0.5)
        } else {
            theme.border()
        })
        .hover(|s| s.bg(theme.hover))
        .child(
            div()
                .text_xs()
                .text_color(if item.is_selected {
                    theme.accent
                } else {
                    theme.text_muted
                })
                .child(item.date.format("%a").to_string()),
        )
        .child(
            div()
                .text_lg()
                .font_weight(if item.is_selected {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if item.is_selected {
                    theme.accent
                } else {
                    theme.text
                })
                .child(item.date.format("%d").to_string()),
        )
}
