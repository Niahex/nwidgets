use crate::components::{Button, ButtonVariant, Dropdown, DropdownOption, TextInput};
use crate::theme::ActiveTheme;
use crate::widgets::tasker::service::TaskService;
use crate::widgets::tasker::types::{Task, TaskStateChanged, TaskStatus};
use chrono::{Datelike, Duration, Local, NaiveDate};
use gpui::prelude::*;
use gpui::*;
use uuid::Uuid;

actions!(tasker, [CloseTasker]);

#[derive(Clone, Copy, PartialEq)]
enum FocusedInput {
    Title,
    Project,
}

#[derive(Clone, Copy, PartialEq)]
enum FormMode {
    Create,
    Edit(Uuid),
}

pub struct TaskWindow {
    task_service: Entity<TaskService>,
    new_task_title: String,
    new_task_project: String,
    new_task_status: TaskStatus,
    new_task_priority: u8,
    new_task_due_date: Option<NaiveDate>,
    pub focus_handle: FocusHandle,
    focused_input: Option<FocusedInput>,
    show_create_modal: bool,
    form_mode: FormMode,
    selected_date: NaiveDate,
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
            new_task_status: TaskStatus::Todo,
            new_task_priority: 5,
            new_task_due_date: None,
            focus_handle: cx.focus_handle(),
            focused_input: None,
            show_create_modal: false,
            form_mode: FormMode::Create,
            selected_date: Local::now().date_naive(),
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

        let mut task = Task::new(self.new_task_title.trim().to_string(), project);
        task.status = self.new_task_status;
        task.priority = self.new_task_priority;
        task.due_date = self.new_task_due_date;

        self.task_service.update(cx, |service, cx| {
            service.add_task(task, cx);
        });

        self.clear_form();
        cx.notify();
    }

    fn update_task(&mut self, task_id: Uuid, cx: &mut Context<Self>) {
        if self.new_task_title.trim().is_empty() {
            return;
        }

        let project = if self.new_task_project.trim().is_empty() {
            None
        } else {
            Some(self.new_task_project.trim().to_string())
        };

        self.task_service.update(cx, |service, cx| {
            service.update_task(
                task_id,
                self.new_task_title.trim().to_string(),
                project,
                self.new_task_status,
                self.new_task_priority,
                self.new_task_due_date,
                cx,
            );
        });

        self.clear_form();
        cx.notify();
    }

    fn clear_form(&mut self) {
        self.new_task_title.clear();
        self.new_task_project.clear();
        self.new_task_status = TaskStatus::Todo;
        self.new_task_priority = 5;
        self.new_task_due_date = None;
        self.focused_input = None;
        self.show_create_modal = false;
        self.form_mode = FormMode::Create;
    }

    fn edit_task(&mut self, task: &Task, cx: &mut Context<Self>) {
        self.new_task_title = task.title.clone();
        self.new_task_project = task.project.clone().unwrap_or_default();
        self.new_task_status = task.status;
        self.new_task_priority = task.priority;
        self.new_task_due_date = task.due_date;
        self.form_mode = FormMode::Edit(task.id);
        self.show_create_modal = true;
        self.focused_input = Some(FocusedInput::Title);
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
                                FocusedInput::Title => {
                                    this.new_task_title.pop();
                                }
                                FocusedInput::Project => {
                                    this.new_task_project.pop();
                                }
                            }
                            cx.notify();
                        } else if event.keystroke.key == "enter" {
                            match this.form_mode {
                                FormMode::Create => this.create_task(cx),
                                FormMode::Edit(task_id) => this.update_task(task_id, cx),
                            }
                        } else if event.keystroke.key == "escape" {
                            this.clear_form();
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
            .child(self.render_date_carousel(theme.clone(), cx))
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
                    .w(px(48.))
                    .h(px(48.))
                    .child(
                        Button::new("add-task-fab")
                            .label(if self.show_create_modal { "×" } else { "+" })
                            .variant(ButtonVariant::Accent)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.show_create_modal = !this.show_create_modal;
                                if this.show_create_modal {
                                    this.focused_input = Some(FocusedInput::Title);
                                    window.focus(&this.focus_handle, cx);
                                }
                                cx.notify();
                            })),
                    ),
            )
    }
}

impl TaskWindow {
    fn render_date_carousel(
        &mut self,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let dates: Vec<NaiveDate> = (-3..=3)
            .map(|i| self.selected_date + Duration::days(i))
            .collect();

        let today = Local::now().date_naive();

        div()
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .p_3()
            .bg(theme.surface)
            .border_b_1()
            .border_color(theme.border())
            .children(dates.iter().map(|date| {
                let date_copy = *date;
                let is_selected = date_copy == self.selected_date;
                let is_today = date_copy == today;

                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(if is_selected {
                        theme.accent.opacity(0.2)
                    } else {
                        theme.surface
                    })
                    .border_1()
                    .border_color(if is_selected {
                        theme.accent
                    } else if is_today {
                        theme.accent.opacity(0.5)
                    } else {
                        theme.border()
                    })
                    .hover(|s| s.bg(theme.hover))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.selected_date = date_copy;
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(if is_selected {
                                theme.accent
                            } else {
                                theme.text_muted
                            })
                            .child(date.format("%a").to_string()),
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_weight(if is_selected {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(if is_selected {
                                theme.accent
                            } else {
                                theme.text
                            })
                            .child(date.format("%d").to_string()),
                    )
            }))
    }

    fn render_create_form(
        &mut self,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let form_title = match self.form_mode {
            FormMode::Create => "Create Task",
            FormMode::Edit(_) => "Edit Task",
        };
        let button_label = match self.form_mode {
            FormMode::Create => "Create Task",
            FormMode::Edit(_) => "Update Task",
        };

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p_4()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text)
                    .child(form_title),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        TextInput::new("task-title-input")
                            .value(self.new_task_title.clone())
                            .placeholder("Task title...")
                            .focused(self.focused_input == Some(FocusedInput::Title))
                            .on_click(|_window, _cx| {}),
                    )
                    .child(
                        TextInput::new("task-project-input")
                            .value(self.new_task_project.clone())
                            .placeholder("Project (optional)")
                            .focused(self.focused_input == Some(FocusedInput::Project))
                            .on_click(|_window, _cx| {}),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted)
                                            .child("Status"),
                                    )
                                    .child(
                                        Dropdown::new(
                                            "task-status-dropdown",
                                            vec![
                                                DropdownOption {
                                                    value: TaskStatus::Todo,
                                                    label: "Todo".into(),
                                                },
                                                DropdownOption {
                                                    value: TaskStatus::InProgress,
                                                    label: "In Progress".into(),
                                                },
                                                DropdownOption {
                                                    value: TaskStatus::Done,
                                                    label: "Done".into(),
                                                },
                                            ],
                                        )
                                        .selected(Some(self.new_task_status))
                                        .on_select(
                                            cx.listener(
                                                |this, status: &TaskStatus, _window, cx| {
                                                    this.new_task_status = *status;
                                                    cx.notify();
                                                },
                                            ),
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted)
                                            .child("Priority"),
                                    )
                                    .child(
                                        Dropdown::new(
                                            "task-priority-dropdown",
                                            vec![
                                                DropdownOption {
                                                    value: 0u8,
                                                    label: "0 - Lowest".into(),
                                                },
                                                DropdownOption {
                                                    value: 1u8,
                                                    label: "1".into(),
                                                },
                                                DropdownOption {
                                                    value: 2u8,
                                                    label: "2".into(),
                                                },
                                                DropdownOption {
                                                    value: 3u8,
                                                    label: "3".into(),
                                                },
                                                DropdownOption {
                                                    value: 4u8,
                                                    label: "4".into(),
                                                },
                                                DropdownOption {
                                                    value: 5u8,
                                                    label: "5 - Medium".into(),
                                                },
                                                DropdownOption {
                                                    value: 6u8,
                                                    label: "6".into(),
                                                },
                                                DropdownOption {
                                                    value: 7u8,
                                                    label: "7".into(),
                                                },
                                                DropdownOption {
                                                    value: 8u8,
                                                    label: "8".into(),
                                                },
                                                DropdownOption {
                                                    value: 9u8,
                                                    label: "9".into(),
                                                },
                                                DropdownOption {
                                                    value: 10u8,
                                                    label: "10 - Highest".into(),
                                                },
                                            ],
                                        )
                                        .selected(Some(self.new_task_priority))
                                        .on_select(
                                            cx.listener(|this, priority: &u8, _window, cx| {
                                                this.new_task_priority = *priority;
                                                cx.notify();
                                            }),
                                        ),
                                    ),
                            ),
                    )
                    .child(
                        Button::new("submit-task-button")
                            .label(button_label)
                            .accent()
                            .on_click(cx.listener(|this, _, _window, cx| match this.form_mode {
                                FormMode::Create => this.create_task(cx),
                                FormMode::Edit(task_id) => this.update_task(task_id, cx),
                            })),
                    ),
            )
    }

    fn render_task_list(
        &mut self,
        tasks: Vec<Task>,
        active_task_id: Option<uuid::Uuid>,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
                let task_clone = task.clone();

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
                                this.bg(theme.green)
                                    .child(div().text_color(theme.bg).text_xs().child("✓"))
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
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(if task.completed {
                                                theme.text_muted
                                            } else {
                                                theme.text
                                            })
                                            .when(task.completed, |this| this.line_through())
                                            .child(task.title.clone()),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded(px(4.))
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
                                            .px_2()
                                            .py_0p5()
                                            .rounded(px(4.))
                                            .text_xs()
                                            .bg(theme.accent.opacity(0.2))
                                            .text_color(theme.accent)
                                            .child(format!("P{}", task.priority)),
                                    ),
                            )
                            .when_some(task.project.as_ref(), |this, project| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.accent)
                                        .child(format!("[{}]", project)),
                                )
                            }),
                    )
                    .child(div().flex().items_center().gap_1().when(
                        task.time_spent_secs > 0,
                        |this| {
                            this.child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(task.format_time_spent()),
                            )
                        },
                    ))
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .text_xs()
                            .bg(theme.surface)
                            .text_color(theme.text)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.hover))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _event, _window, cx| {
                                    cx.stop_propagation();
                                    this.edit_task(&task_clone, cx);
                                }),
                            )
                            .child("Edit"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .text_xs()
                            .bg(theme.red.opacity(0.2))
                            .text_color(theme.red)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.red.opacity(0.3)))
                            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                cx.stop_propagation();
                                task_service_delete.update(cx, |service, cx| {
                                    service.remove_task(task_id, cx);
                                });
                            })
                            .child("Delete"),
                    )
            }))
    }
}
