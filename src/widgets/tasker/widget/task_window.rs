use crate::components::Button;
use crate::theme::ActiveTheme;
use crate::widgets::tasker::service::TaskService;
use crate::widgets::tasker::types::{Task, TaskStateChanged};
use gpui::prelude::*;
use gpui::*;

actions!(tasker, [CloseTasker]);

#[derive(Clone, Copy, PartialEq)]
enum FocusedInput {
    Title,
    Project,
}

pub struct TaskWindow {
    task_service: Entity<TaskService>,
    new_task_title: String,
    new_task_project: String,
    pub focus_handle: FocusHandle,
    focused_input: Option<FocusedInput>,
    show_create_modal: bool,
}

impl TaskWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let task_service = TaskService::global(cx);

        cx.subscribe(
            &task_service,
            |_this, _service, _event: &TaskStateChanged, cx| {
                cx.notify();
            },
        )
        .detach();

        Self {
            task_service,
            new_task_title: String::new(),
            new_task_project: String::new(),
            focus_handle: cx.focus_handle(),
            focused_input: None,
            show_create_modal: false,
        }
    }

    fn create_task(&mut self, cx: &mut Context<Self>) {
        if self.new_task_title.trim().is_empty() {
            return;
        }

        let project = if self.new_task_project.trim().is_empty() {
            None
        } else {
            Some(self.new_task_project.trim().to_string())
        };

        let task = Task::new(self.new_task_title.trim().to_string(), project);

        self.task_service.update(cx, |service, cx| {
            service.add_task(task, cx);
        });

        self.new_task_title.clear();
        self.new_task_project.clear();
        self.focused_input = None;
        self.show_create_modal = false;
        cx.notify();
    }
}

impl Render for TaskWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let tasks = self.task_service.read(cx).tasks();
        let active_task_id = self.task_service.read(cx).active_task_id();
        let focus_handle = self.focus_handle.clone();

        div()
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if this.show_create_modal {
                    if let Some(focused) = this.focused_input {
                        if event.keystroke.key == "backspace" {
                            match focused {
                                FocusedInput::Title => { this.new_task_title.pop(); },
                                FocusedInput::Project => { this.new_task_project.pop(); },
                            }
                            cx.notify();
                        } else if event.keystroke.key == "enter" {
                            this.create_task(cx);
                        } else if event.keystroke.key == "escape" {
                            this.show_create_modal = false;
                            this.focused_input = None;
                            cx.notify();
                        } else if event.keystroke.key == "tab" {
                            this.focused_input = match focused {
                                FocusedInput::Title => Some(FocusedInput::Project),
                                FocusedInput::Project => Some(FocusedInput::Title),
                            };
                            cx.notify();
                        } else if let Some(key_char) = &event.keystroke.key_char {
                            if key_char.len() == 1 {
                                let target = match focused {
                                    FocusedInput::Title => &mut this.new_task_title,
                                    FocusedInput::Project => &mut this.new_task_project,
                                };
                                target.push_str(key_char);
                                cx.notify();
                            }
                        }
                    }
                }
            }))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.bg)
            .on_action(cx.listener(|this, _: &CloseTasker, _window, cx| {
                this.task_service.update(cx, |service, cx| {
                    service.toggle_window(cx);
                });
            }))
            .on_mouse_down_out(cx.listener(|this, _, _window, cx| {
                let visible = this.task_service.read(cx).window_visible();
                if visible {
                    this.task_service.update(cx, |service, cx| {
                        service.toggle_window(cx);
                    });
                }
            }))
            .when(self.show_create_modal, |this| {
                this.child(self.render_create_form(theme.clone(), cx))
            })
            .when(!self.show_create_modal, |this| {
                this.child(self.render_task_list(tasks.clone(), active_task_id, theme.clone(), cx))
            })
            .child(
                div()
                    .absolute()
                    .bottom_4()
                    .right_4()
                    .child(
                        div()
                            .w(px(48.))
                            .h(px(48.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(theme.accent)
                            .rounded(px(24.))
                            .text_color(theme.bg)
                            .text_xl()
                            .cursor_pointer()
                            .hover(|style| style.bg(theme.accent.opacity(0.8)))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                this.show_create_modal = !this.show_create_modal;
                                if this.show_create_modal {
                                    this.focused_input = Some(FocusedInput::Title);
                                    window.focus(&this.focus_handle, cx);
                                }
                                cx.notify();
                            }))
                            .child(if self.show_create_modal { "×" } else { "+" }),
                    ),
            )
    }
}

impl TaskWindow {
    fn render_create_form(&mut self, theme: crate::theme::Theme, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p_4()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .px_3()
                            .py_2()
                            .bg(theme.surface)
                            .rounded(px(4.))
                            .border_1()
                            .border_color(if self.focused_input == Some(FocusedInput::Title) {
                                theme.accent
                            } else {
                                theme.border()
                            })
                            .cursor_text()
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                this.focused_input = Some(FocusedInput::Title);
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                            }))
                            .when(self.new_task_title.is_empty(), |this| {
                                this.text_color(theme.text_muted).child("Task title...")
                            })
                            .when(!self.new_task_title.is_empty(), |this| {
                                this.text_color(theme.text).child(self.new_task_title.clone())
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .px_3()
                            .py_2()
                            .bg(theme.surface)
                            .rounded(px(4.))
                            .border_1()
                            .border_color(if self.focused_input == Some(FocusedInput::Project) {
                                theme.accent
                            } else {
                                theme.border()
                            })
                            .cursor_text()
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                this.focused_input = Some(FocusedInput::Project);
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                            }))
                            .when(self.new_task_project.is_empty(), |this| {
                                this.text_color(theme.text_muted).child("Project (optional)")
                            })
                            .when(!self.new_task_project.is_empty(), |this| {
                                this.text_color(theme.text).child(self.new_task_project.clone())
                            }),
                    )
                    .child(
                        Button::new("add-task-button")
                            .label("Create Task")
                            .accent()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.create_task(cx);
                            })),
                    ),
            )
    }

    fn render_task_list(&mut self, tasks: Vec<Task>, active_task_id: Option<uuid::Uuid>, theme: crate::theme::Theme, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .p_4()
            .gap_2()
            .children(tasks.iter().map(|task| {
                        let task_id = task.id;
                        let is_active = active_task_id == Some(task_id);
                        let task_service = self.task_service.clone();
                        let task_service_toggle = self.task_service.clone();
                        let task_service_delete = self.task_service.clone();

                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .p_2()
                            .rounded(px(6.))
                            .bg(if is_active {
                                theme.accent.opacity(0.2)
                            } else {
                                theme.surface
                            })
                            .border_1()
                            .border_color(if is_active {
                                theme.accent
                            } else {
                                theme.border()
                            })
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, move |_, _window, cx| {
                                task_service.update(cx, |service, cx| {
                                    service.select_task(Some(task_id), cx);
                                });
                            })
                            .child(
                                div()
                                    .w(px(16.))
                                    .h(px(16.))
                                    .rounded(px(3.))
                                    .border_1()
                                    .border_color(theme.border())
                                    .cursor_pointer()
                                    .when(task.completed, |this| {
                                        this.bg(theme.green).child(
                                            div()
                                                .text_color(theme.bg)
                                                .text_xs()
                                                .child("✓"),
                                        )
                                    })
                                    .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                        cx.stop_propagation();
                                        task_service_toggle.update(cx, |service, cx| {
                                            service.toggle_task_completed(task_id, cx);
                                        });
                                    }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(if task.completed {
                                                theme.text_muted
                                            } else {
                                                theme.text
                                            })
                                            .when(task.completed, |this| {
                                                this.line_through()
                                            })
                                            .child(task.title.clone()),
                                    )
                                    .when(task.project.is_some(), |this| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.accent)
                                                .child(format!("[{}]", task.project.as_ref().unwrap())),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .when(task.time_spent_secs > 0, |this| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.text_muted)
                                                .child(task.format_time_spent()),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(px(4.))
                                    .bg(theme.red.opacity(0.2))
                                    .text_color(theme.red)
                                    .text_xs()
                                    .cursor_pointer()
                                    .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                        task_service_delete.update(cx, |service, cx| {
                                            service.remove_task(task_id, cx);
                                        });
                                    })
                                    .child("Delete"),
                            )
                    }))
    }
}
