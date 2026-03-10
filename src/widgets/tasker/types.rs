use chrono::{DateTime, Utc};
use gpui::{EventEmitter, SharedString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub project: Option<String>,
    pub estimated_pomodoros: u32,
    pub completed_pomodoros: u32,
    pub created_at: DateTime<Utc>,
    pub completed: bool,
}

impl Task {
    pub fn new(title: String, project: Option<String>, estimated_pomodoros: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            project,
            estimated_pomodoros,
            completed_pomodoros: 0,
            created_at: Utc::now(),
            completed: false,
        }
    }

    pub fn progress(&self) -> f32 {
        if self.estimated_pomodoros == 0 {
            return 0.0;
        }
        (self.completed_pomodoros as f32 / self.estimated_pomodoros as f32).min(1.0)
    }

    pub fn display_title(&self) -> SharedString {
        if let Some(ref project) = self.project {
            format!("[{}] {}", project, self.title).into()
        } else {
            self.title.clone().into()
        }
    }
}

#[derive(Clone)]
pub struct TaskStateChanged;

#[derive(Clone)]
pub struct TaskSelected {
    pub task_id: Option<Uuid>,
}

#[derive(Clone)]
pub struct TaskWindowToggled;

pub trait TaskEvents: EventEmitter<TaskStateChanged> + EventEmitter<TaskSelected> + EventEmitter<TaskWindowToggled> {}
