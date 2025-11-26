use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub due_date: String,
    pub priority: String,
    pub project: String,
    pub category: String,
    pub completed: bool,
    pub created_at: String,
}

impl Task {
    pub fn new(
        name: String,
        description: String,
        due_date: String,
        priority: String,
        project: String,
        category: String,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Local::now().to_rfc3339();

        Self {
            id,
            name,
            description,
            due_date,
            priority,
            project,
            category,
            completed: false,
            created_at,
        }
    }
}

pub struct TasksService;

impl TasksService {
    fn get_tasks_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/nwidgets/tasks.json")
    }

    pub fn load_tasks() -> Vec<Task> {
        let path = Self::get_tasks_file_path();

        if !path.exists() {
            return Vec::new();
        }

        match fs::read_to_string(&path) {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_else(|e| {
                    eprintln!("[TASKS] Error parsing tasks file: {}", e);
                    Vec::new()
                })
            }
            Err(e) => {
                eprintln!("[TASKS] Error reading tasks file: {}", e);
                Vec::new()
            }
        }
    }

    pub fn save_tasks(tasks: &[Task]) -> Result<(), String> {
        let path = Self::get_tasks_file_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(tasks).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;

        println!("[TASKS] Tasks saved to {:?}", path);
        Ok(())
    }

    pub fn add_task(task: Task) -> Result<(), String> {
        let mut tasks = Self::load_tasks();
        tasks.push(task);
        Self::save_tasks(&tasks)
    }

    pub fn update_task(task: Task) -> Result<(), String> {
        let mut tasks = Self::load_tasks();

        if let Some(index) = tasks.iter().position(|t| t.id == task.id) {
            tasks[index] = task;
            Self::save_tasks(&tasks)
        } else {
            Err("Task not found".to_string())
        }
    }

    pub fn delete_task(task_id: &str) -> Result<(), String> {
        let mut tasks = Self::load_tasks();
        tasks.retain(|t| t.id != task_id);
        Self::save_tasks(&tasks)
    }

    pub fn get_tasks_for_date(date: &str) -> Vec<Task> {
        Self::load_tasks()
            .into_iter()
            .filter(|t| t.due_date == date)
            .collect()
    }
}
