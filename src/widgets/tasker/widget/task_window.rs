mod carousel;
mod form;
mod list;

use crate::components::Button;
use crate::components::ButtonVariant;
use crate::theme::ActiveTheme;
use crate::widgets::tasker::service::TaskService;
use crate::widgets::tasker::types::{Task, TaskStateChanged};
use chrono::Local;
use gpui::prelude::*;
use gpui::*;

pub use form::{FocusedInput, FormMode, FormState};

actions!(tasker, [CloseTasker]);

pub struct TaskWindow {
    task_service: Entity<TaskService>,
    pub focus_handle: FocusHandle,
    show_create_modal: bool,
    form_state: FormState,
    selected_date: chrono::NaiveDate,
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
            focus_handle: cx.focus_handle(),
            show_create_modal: false,
            form_state: FormState::new(),
            selected_date: Local::now().date_naive(),
        }
    }

    fn create_task(&mut self, cx: &mut Context<Self>) {
        if self.form_state.title.trim().is_empty() {
            return;
        }

        let project = if self.form_state.project.trim().is_empty() {
            None
        } else {
            Some(self.form_state.project.trim().to_string())
        };

        let mut task = Task::new(self.form_state.title.trim().to_string(), project);
        task.status = self.form_state.status;
        task.priority = self.form_state.priority;
        task.due_date = self.form_state.due_date;

        self.task_service.update(cx, |service, cx| {
            service.add_task(task, cx);
        });

        self.form_state.clear();
        self.show_create_modal = false;
        cx.notify();
    }

    fn update_task(&mut self, task_id: uuid::Uuid, cx: &mut Context<Self>) {
        if self.form_state.title.trim().is_empty() {
            return;
        }

        let project = if self.form_state.project.trim().is_empty() {
            None
        } else {
            Some(self.form_state.project.trim().to_string())
        };

        self.task_service.update(cx, |service, cx| {
            service.update_task(
                task_id,
                self.form_state.title.trim().to_string(),
                project,
                self.form_state.status,
                self.form_state.priority,
                self.form_state.due_date,
                cx,
            );
        });

        self.form_state.clear();
        self.show_create_modal = false;
        cx.notify();
    }

    fn edit_task(&mut self, task: &Task, cx: &mut Context<Self>) {
        self.form_state.set_edit_mode(
            task.id,
            task.title.clone(),
            task.project.clone(),
            task.status,
            task.priority,
            task.due_date,
        );
        self.show_create_modal = true;
        cx.notify();
    }

    fn render_date_carousel(
        &mut self,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let date_items = carousel::build_date_items(self.selected_date);

        div()
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .p_3()
            .bg(theme.surface)
            .border_b_1()
            .border_color(theme.border())
            .children(date_items.iter().map(|item| {
                let date = item.date;
                carousel::render_date_item(item, &theme).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.selected_date = date;
                        cx.notify();
                    }),
                )
            }))
    }

    fn render_create_form(
        &mut self,
        theme: crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p_4()
            .gap_4()
            .child(form::render_form_header(self.form_state.mode, &theme))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(form::render_title_input(
                        &self.form_state.title,
                        self.form_state.focused_input == Some(FocusedInput::Title),
                    ))
                    .child(form::render_project_input(
                        &self.form_state.project,
                        self.form_state.focused_input == Some(FocusedInput::Project),
                    ))
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
                                        form::render_status_dropdown(self.form_state.status)
                                            .on_select(cx.listener(|this, status, _window, cx| {
                                                this.form_state.status = *status;
                                                cx.notify();
                                            })),
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
                                        form::render_priority_dropdown(self.form_state.priority)
                                            .on_select(cx.listener(
                                                |this, priority, _window, cx| {
                                                    this.form_state.priority = *priority;
                                                    cx.notify();
                                                },
                                            )),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child("Due Date"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(form::render_due_date_display(
                                        self.form_state.due_date,
                                        &theme,
                                    ))
                                    .child(form::render_set_due_date_button(&theme).on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.form_state.due_date = Some(this.selected_date);
                                            cx.notify();
                                        }),
                                    ))
                                    .when(self.form_state.due_date.is_some(), |this| {
                                        this.child(
                                            form::render_clear_due_date_button(&theme)
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(|this, _, _, cx| {
                                                        this.form_state.due_date = None;
                                                        cx.notify();
                                                    }),
                                                ),
                                        )
                                    }),
                            ),
                    )
                    .child(
                        form::render_submit_button(self.form_state.mode).on_click(cx.listener(
                            |this, _, _window, cx| match this.form_state.mode {
                                FormMode::Create => this.create_task(cx),
                                FormMode::Edit(task_id) => this.update_task(task_id, cx),
                            },
                        )),
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
        let filtered_tasks = list::filter_tasks_by_date(tasks, self.selected_date);

        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .p_4()
            .gap_2()
            .when(filtered_tasks.is_empty(), |this| {
                this.child(list::render_empty_state(self.selected_date, &theme))
            })
            .children(filtered_tasks.iter().map(|task| {
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
                    .child(list::render_checkbox(task.completed, &theme).on_mouse_down(
                        MouseButton::Left,
                        move |_event, _window, cx| {
                            cx.stop_propagation();
                            task_service_toggle.update(cx, |service, cx| {
                                service.toggle_task_completed(task_id, cx);
                            });
                        },
                    ))
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
                                    .child(list::render_status_badge(task.status, &theme))
                                    .child(list::render_priority_badge(task.priority, &theme))
                                    .when_some(task.due_date, |this, due_date| {
                                        this.child(list::render_due_date_badge(
                                            due_date,
                                            task.completed,
                                            &theme,
                                        ))
                                    }),
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
                    .child(list::render_edit_button(&theme).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            cx.stop_propagation();
                            this.edit_task(&task_clone, cx);
                        }),
                    ))
                    .child(list::render_delete_button(&theme).on_mouse_down(
                        MouseButton::Left,
                        move |_event, _window, cx| {
                            cx.stop_propagation();
                            task_service_delete.update(cx, |service, cx| {
                                service.remove_task(task_id, cx);
                            });
                        },
                    ))
            }))
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
                    if let Some(focused) = this.form_state.focused_input {
                        if event.keystroke.key == "backspace" {
                            match focused {
                                FocusedInput::Title => {
                                    this.form_state.title.pop();
                                }
                                FocusedInput::Project => {
                                    this.form_state.project.pop();
                                }
                            }
                            cx.notify();
                        } else if event.keystroke.key == "enter" {
                            match this.form_state.mode {
                                FormMode::Create => this.create_task(cx),
                                FormMode::Edit(task_id) => this.update_task(task_id, cx),
                            }
                        } else if event.keystroke.key == "escape" {
                            this.form_state.clear();
                            this.show_create_modal = false;
                            cx.notify();
                        } else if event.keystroke.key == "tab" {
                            this.form_state.focused_input = match focused {
                                FocusedInput::Title => Some(FocusedInput::Project),
                                FocusedInput::Project => Some(FocusedInput::Title),
                            };
                            cx.notify();
                        } else if let Some(key_char) = &event.keystroke.key_char {
                            if key_char.len() == 1 {
                                let target = match focused {
                                    FocusedInput::Title => &mut this.form_state.title,
                                    FocusedInput::Project => &mut this.form_state.project,
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
                                    this.form_state.focused_input = Some(FocusedInput::Title);
                                    window.focus(&this.focus_handle, cx);
                                } else {
                                    this.form_state.clear();
                                }
                                cx.notify();
                            })),
                    ),
            )
    }
}
