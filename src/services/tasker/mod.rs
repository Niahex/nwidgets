use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate, Local};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyTaskList {
    pub date: NaiveDate,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub completed: bool,
    pub date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskerService {
    state: Arc<RwLock<TaskerState>>,
}

#[derive(Default)]
struct TaskerState {
    // Map of date (YYYY-MM-DD) to list of tasks for that day
    daily_tasks: HashMap<String, Vec<Task>>,
}

impl TaskerService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(TaskerState::default())),
        };
        
        service.load_from_disk();
        service
    }

    pub fn get_tasks_for_date(&self, date: NaiveDate) -> Vec<Task> {
        let state = self.state.read();
        let date_key = date.format("%Y-%m-%d").to_string();
        state.daily_tasks.get(&date_key).cloned().unwrap_or_default()
    }

    pub fn get_date_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<DailyTaskList> {
        let state = self.state.read();
        let mut result = Vec::new();
        
        let mut current = start;
        while current <= end {
            let date_key = current.format("%Y-%m-%d").to_string();
            let tasks = state.daily_tasks.get(&date_key).cloned().unwrap_or_default();
            result.push(DailyTaskList {
                date: current,
                tasks,
            });
            current = current.succ_opt().unwrap_or(current);
        }
        
        result
    }

    pub fn add_task(&self, date: NaiveDate, task_name: String) -> Task {
        let now = Utc::now();
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            name: task_name,
            description: None,
            completed: false,
            date,
            created_at: now,
            updated_at: now,
        };

        let date_key = date.format("%Y-%m-%d").to_string();
        let mut state = self.state.write();
        state.daily_tasks.entry(date_key).or_insert_with(Vec::new).push(task.clone());
        
        drop(state);
        self.save_to_disk();
        task
    }

    pub fn toggle_task(&self, task_id: &str) -> Option<bool> {
        let mut state = self.state.write();
        
        for tasks in state.daily_tasks.values_mut() {
            for task in tasks.iter_mut() {
                if task.id == task_id {
                    task.completed = !task.completed;
                    task.updated_at = Utc::now();
                    let completed = task.completed;
                    drop(state);
                    self.save_to_disk();
                    return Some(completed);
                }
            }
        }
        None
    }

    pub fn delete_task(&self, task_id: &str) -> bool {
        let mut state = self.state.write();
        
        for tasks in state.daily_tasks.values_mut() {
            if let Some(pos) = tasks.iter().position(|t| t.id == task_id) {
                tasks.remove(pos);
                drop(state);
                self.save_to_disk();
                return true;
            }
        }
        false
    }

    pub fn get_today() -> NaiveDate {
        Local::now().date_naive()
    }

    fn load_from_disk(&self) {
        let path = Self::get_data_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(daily_tasks) = serde_json::from_str::<HashMap<String, Vec<Task>>>(&content) {
                let mut state = self.state.write();
                state.daily_tasks = daily_tasks;
                log::info!("Loaded {} days of tasks from disk", state.daily_tasks.len());
            }
        }
    }

    fn save_to_disk(&self) {
        let state = self.state.read();
        
        if let Ok(json) = serde_json::to_string_pretty(&state.daily_tasks) {
            let path = Self::get_data_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, json);
        }
    }

    fn get_data_path() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home)
            .join(".local/share/nwidgets/tasker.json")
    }
}
