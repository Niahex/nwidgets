use crate::theme::Theme;
use crate::widgets::tasker::types::{Task, TaskStatus};
use chrono::{Local, NaiveDate};
use gpui::prelude::*;
use gpui::*;

pub struct TaskItem {
    pub task: Task,
    pub is_active: bool,
}

pub fn filter_tasks_by_date(tasks: Vec<Task>, selected_date: NaiveDate) -> Vec<Task> {
    tasks
        .into_iter()
        .filter(|task| task.due_date.map(|d| d == selected_date).unwrap_or(false))
        .collect()
}

pub fn render_empty_state(selected_date: NaiveDate, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .flex_1()
        .text_sm()
        .text_color(theme.text_muted)
        .child(format!("No tasks for {}", selected_date.format("%Y-%m-%d")))
}

pub fn render_checkbox(completed: bool, theme: &Theme) -> Div {
    div()
        .w(px(16.))
        .h(px(16.))
        .rounded(px(3.))
        .border_1()
        .border_color(theme.border())
        .cursor_pointer()
        .when(completed, |this| {
            this.bg(theme.green)
                .child(div().text_color(theme.bg).text_xs().child("✓"))
        })
}

pub fn render_status_badge(status: TaskStatus, theme: &Theme) -> impl IntoElement {
    div()
        .px_2()
        .py_0p5()
        .rounded(px(4.))
        .text_xs()
        .bg(match status {
            TaskStatus::Todo => theme.surface,
            TaskStatus::InProgress => theme.accent.opacity(0.2),
            TaskStatus::Done => theme.green.opacity(0.2),
        })
        .text_color(match status {
            TaskStatus::Todo => theme.text_muted,
            TaskStatus::InProgress => theme.accent,
            TaskStatus::Done => theme.green,
        })
        .child(status.as_str())
}

pub fn render_priority_badge(priority: u8, theme: &Theme) -> impl IntoElement {
    div()
        .px_2()
        .py_0p5()
        .rounded(px(4.))
        .text_xs()
        .bg(theme.accent.opacity(0.2))
        .text_color(theme.accent)
        .child(format!("P{}", priority))
}

pub fn render_due_date_badge(
    due_date: NaiveDate,
    completed: bool,
    theme: &Theme,
) -> impl IntoElement {
    let today = Local::now().date_naive();
    let is_overdue = due_date < today && !completed;
    let is_today = due_date == today;

    div()
        .px_2()
        .py_0p5()
        .rounded(px(4.))
        .text_xs()
        .bg(if is_overdue {
            theme.red.opacity(0.2)
        } else if is_today {
            theme.orange.opacity(0.2)
        } else {
            theme.surface
        })
        .text_color(if is_overdue {
            theme.red
        } else if is_today {
            theme.orange
        } else {
            theme.text_muted
        })
        .child(due_date.format("%m/%d").to_string())
}

pub fn render_edit_button(theme: &Theme) -> Div {
    div()
        .px_3()
        .py_1p5()
        .rounded_md()
        .text_xs()
        .bg(theme.surface)
        .text_color(theme.text)
        .cursor_pointer()
        .hover(|s| s.bg(theme.hover))
        .child("Edit")
}

pub fn render_delete_button(theme: &Theme) -> Div {
    div()
        .px_3()
        .py_1p5()
        .rounded_md()
        .text_xs()
        .bg(theme.red.opacity(0.2))
        .text_color(theme.red)
        .cursor_pointer()
        .hover(|s| s.bg(theme.red.opacity(0.3)))
        .child("Delete")
}
