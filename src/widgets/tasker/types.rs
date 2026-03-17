use chrono::{DateTime, Utc};
use gpui::{EventEmitter, SharedString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub project: Option<String>,
    pub time_spent_secs: u64,
    pub created_at: DateTime<Utc>,
    pub completed: bool,
}

impl Task {
    pub fn new(title: String, project: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            project,
            time_spent_secs: 0,
            created_at: Utc::now(),
            completed: false,
        }
    }

    pub fn format_time_spent(&self) -> SharedString {
        let total = self.time_spent_secs;
        let hours = total / 3600;
        let mins = (total % 3600) / 60;
        if hours > 0 {
            format!("{hours}h{mins:02}m").into()
        } else {
            format!("{mins}m").into()
        }
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
