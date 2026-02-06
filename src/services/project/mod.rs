use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tasks: Vec<Task>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub completed: bool,
    pub subtasks: Vec<Task>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub parent_id: Option<String>,
}

#[derive(Clone)]
pub struct ProjectService {
    state: Arc<RwLock<ProjectState>>,
}

#[derive(Default)]
struct ProjectState {
    projects: HashMap<String, Project>,
}

impl ProjectService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(ProjectState::default())),
        };
        
        service.load_from_disk();
        service
    }

    pub fn get_all_projects(&self) -> Vec<Project> {
        let state = self.state.read();
        state.projects.values().cloned().collect()
    }

    pub fn get_project(&self, id: &str) -> Option<Project> {
        self.state.read().projects.get(id).cloned()
    }

    pub fn create_project(&self, name: String, description: Option<String>) -> Project {
        let now = Utc::now();
        let project = Project {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            tasks: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        self.state.write().projects.insert(project.id.clone(), project.clone());
        self.save_to_disk();
        project
    }

    pub fn add_task(&self, project_id: &str, task_name: String, parent_id: Option<String>) -> Option<Task> {
        let mut state = self.state.write();
        let project = state.projects.get_mut(project_id)?;

        let now = Utc::now();
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            name: task_name,
            description: None,
            completed: false,
            subtasks: Vec::new(),
            created_at: now,
            updated_at: now,
            parent_id: parent_id.clone(),
        };

        if let Some(parent_id) = parent_id {
            Self::add_subtask_recursive(&mut project.tasks, &parent_id, task.clone())?;
        } else {
            project.tasks.push(task.clone());
        }

        project.updated_at = now;
        self.save_to_disk();
        Some(task)
    }

    fn add_subtask_recursive(tasks: &mut Vec<Task>, parent_id: &str, new_task: Task) -> Option<()> {
        for task in tasks.iter_mut() {
            if task.id == parent_id {
                task.subtasks.push(new_task);
                return Some(());
            }
            if Self::add_subtask_recursive(&mut task.subtasks, parent_id, new_task.clone()).is_some() {
                return Some(());
            }
        }
        None
    }

    pub fn toggle_task(&self, project_id: &str, task_id: &str) -> Option<bool> {
        let mut state = self.state.write();
        let project = state.projects.get_mut(project_id)?;
        
        let completed = Self::toggle_task_recursive(&mut project.tasks, task_id)?;
        project.updated_at = Utc::now();
        self.save_to_disk();
        Some(completed)
    }

    fn toggle_task_recursive(tasks: &mut Vec<Task>, task_id: &str) -> Option<bool> {
        for task in tasks.iter_mut() {
            if task.id == task_id {
                task.completed = !task.completed;
                task.updated_at = Utc::now();
                return Some(task.completed);
            }
            if let Some(completed) = Self::toggle_task_recursive(&mut task.subtasks, task_id) {
                return Some(completed);
            }
        }
        None
    }

    fn load_from_disk(&self) {
        let path = Self::get_data_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(projects) = serde_json::from_str::<Vec<Project>>(&content) {
                let mut state = self.state.write();
                for project in projects {
                    state.projects.insert(project.id.clone(), project);
                }
                log::info!("Loaded {} projects from disk", state.projects.len());
            }
        }
    }

    fn save_to_disk(&self) {
        let state = self.state.read();
        let projects: Vec<Project> = state.projects.values().cloned().collect();
        
        if let Ok(json) = serde_json::to_string_pretty(&projects) {
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
            .join(".local/share/nwidgets/projects.json")
    }
}
