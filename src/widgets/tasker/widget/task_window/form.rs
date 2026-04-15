use crate::components::{Button, ButtonVariant, Dropdown, DropdownOption, TextInput};
use crate::theme::Theme;
use crate::widgets::tasker::types::TaskStatus;
use chrono::NaiveDate;
use gpui::prelude::*;
use gpui::*;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
pub enum FocusedInput {
    Title,
    Project,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FormMode {
    Create,
    Edit(Uuid),
}

pub struct FormState {
    pub title: String,
    pub project: String,
    pub status: TaskStatus,
    pub priority: u8,
    pub due_date: Option<NaiveDate>,
    pub focused_input: Option<FocusedInput>,
    pub mode: FormMode,
}

impl FormState {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            project: String::new(),
            status: TaskStatus::Todo,
            priority: 5,
            due_date: None,
            focused_input: None,
            mode: FormMode::Create,
        }
    }

    pub fn clear(&mut self) {
        self.title.clear();
        self.project.clear();
        self.status = TaskStatus::Todo;
        self.priority = 5;
        self.due_date = None;
        self.focused_input = None;
        self.mode = FormMode::Create;
    }

    pub fn set_edit_mode(
        &mut self,
        task_id: Uuid,
        title: String,
        project: Option<String>,
        status: TaskStatus,
        priority: u8,
        due_date: Option<NaiveDate>,
    ) {
        self.title = title;
        self.project = project.unwrap_or_default();
        self.status = status;
        self.priority = priority;
        self.due_date = due_date;
        self.mode = FormMode::Edit(task_id);
        self.focused_input = Some(FocusedInput::Title);
    }
}

pub fn get_form_title(mode: FormMode) -> &'static str {
    match mode {
        FormMode::Create => "Create Task",
        FormMode::Edit(_) => "Edit Task",
    }
}

pub fn get_button_label(mode: FormMode) -> &'static str {
    match mode {
        FormMode::Create => "Create Task",
        FormMode::Edit(_) => "Update Task",
    }
}

pub fn render_form_header(mode: FormMode, theme: &Theme) -> impl IntoElement {
    div()
        .text_lg()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.text)
        .child(get_form_title(mode))
}

pub fn render_title_input(title: &str, focused: bool) -> impl IntoElement {
    TextInput::new("task-title-input")
        .value(title.to_string())
        .placeholder("Task title...")
        .focused(focused)
        .on_click(|_window, _cx| {})
}

pub fn render_project_input(project: &str, focused: bool) -> impl IntoElement {
    TextInput::new("task-project-input")
        .value(project.to_string())
        .placeholder("Project (optional)")
        .focused(focused)
        .on_click(|_window, _cx| {})
}

pub fn render_status_dropdown(selected: TaskStatus) -> Dropdown<TaskStatus> {
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
    .selected(Some(selected))
}

pub fn render_priority_dropdown(selected: u8) -> Dropdown<u8> {
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
    .selected(Some(selected))
}

pub fn render_due_date_display(due_date: Option<NaiveDate>, theme: &Theme) -> impl IntoElement {
    div()
        .flex_1()
        .px_2()
        .py_1p5()
        .rounded_md()
        .bg(theme.surface)
        .text_xs()
        .text_color(if due_date.is_some() {
            theme.text
        } else {
            theme.text_muted
        })
        .child(
            due_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "No due date".to_string()),
        )
}

pub fn render_set_due_date_button(theme: &Theme) -> Div {
    div()
        .px_3()
        .py_1p5()
        .rounded_md()
        .text_xs()
        .bg(theme.accent.opacity(0.2))
        .text_color(theme.accent)
        .cursor_pointer()
        .hover(|s| s.bg(theme.accent.opacity(0.3)))
        .child("Set to selected")
}

pub fn render_clear_due_date_button(theme: &Theme) -> Div {
    div()
        .px_3()
        .py_1p5()
        .rounded_md()
        .text_xs()
        .bg(theme.red.opacity(0.2))
        .text_color(theme.red)
        .cursor_pointer()
        .hover(|s| s.bg(theme.red.opacity(0.3)))
        .child("Clear")
}

pub fn render_submit_button(mode: FormMode) -> Button {
    Button::new("submit-task-button")
        .label(get_button_label(mode))
        .variant(ButtonVariant::Accent)
}
