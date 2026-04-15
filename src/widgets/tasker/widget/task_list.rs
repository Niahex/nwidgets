use crate::theme::ActiveTheme;
use crate::widgets::tasker::service::TaskService;
use crate::widgets::tasker::types::TaskStatus;
use gpui::prelude::*;
use gpui::*;

pub struct TaskListWidget {
    task_service: Entity<TaskService>,
}

impl TaskListWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let task_service = TaskService::global(cx);
        Self { task_service }
    }
}

impl Render for TaskListWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let tasks = self.task_service.read(cx).tasks();
        let active_task = self.task_service.read(cx).active_task();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .when_some(active_task, |this, task| {
                this.child(
                    div()
                        .p_2()
                        .bg(theme.accent.opacity(0.2))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(theme.accent)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .flex_1()
                                        .text_sm()
                                        .text_color(theme.text)
                                        .child(format!("Active: {}", task.title)),
                                )
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(3.))
                                        .text_xs()
                                        .bg(match task.status {
                                            TaskStatus::Todo => theme.surface,
                                            TaskStatus::InProgress => theme.accent.opacity(0.3),
                                            TaskStatus::Done => theme.green.opacity(0.3),
                                        })
                                        .text_color(match task.status {
                                            TaskStatus::Todo => theme.text_muted,
                                            TaskStatus::InProgress => theme.accent,
                                            TaskStatus::Done => theme.green,
                                        })
                                        .child(task.status.as_str()),
                                )
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(3.))
                                        .text_xs()
                                        .bg(theme.accent.opacity(0.3))
                                        .text_color(theme.accent)
                                        .child(format!("P{}", task.priority)),
                                ),
                        )
                        .when(task.time_spent_secs > 0, |this| {
                            this.child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(task.format_time_spent()),
                            )
                        }),
                )
            })
            .children(tasks.iter().take(5).map(|task| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .p_1()
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(theme.text)
                            .child(task.title.clone()),
                    )
                    .child(
                        div()
                            .px_1p5()
                            .py_0p5()
                            .rounded(px(3.))
                            .text_xs()
                            .bg(match task.status {
                                TaskStatus::Todo => theme.surface,
                                TaskStatus::InProgress => theme.accent.opacity(0.2),
                                TaskStatus::Done => theme.green.opacity(0.2),
                            })
                            .text_color(match task.status {
                                TaskStatus::Todo => theme.text_muted,
                                TaskStatus::InProgress => theme.accent,
                                TaskStatus::Done => theme.green,
                            })
                            .child(task.status.as_str()),
                    )
                    .child(
                        div()
                            .px_1p5()
                            .py_0p5()
                            .rounded(px(3.))
                            .text_xs()
                            .bg(theme.accent.opacity(0.2))
                            .text_color(theme.accent)
                            .child(format!("P{}", task.priority)),
                    )
                    .when(task.time_spent_secs > 0, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(task.format_time_spent()),
                        )
                    })
            }))
    }
}
