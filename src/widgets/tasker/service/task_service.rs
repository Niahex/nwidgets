use crate::widgets::tasker::types::{Task, TaskSelected, TaskStateChanged, TaskWindowToggled};
use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

pub struct TaskService {
    tasks: Arc<RwLock<Vec<Task>>>,
    active_task_id: Arc<RwLock<Option<Uuid>>>,
    window_visible: Arc<RwLock<bool>>,
    storage_path: PathBuf,
}

impl EventEmitter<TaskStateChanged> for TaskService {}
impl EventEmitter<TaskSelected> for TaskService {}
impl EventEmitter<TaskWindowToggled> for TaskService {}

impl TaskService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let storage_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nwidgets")
            .join("tasker.json");

        let tasks = Self::load_tasks(&storage_path);

        Self {
            tasks: Arc::new(RwLock::new(tasks)),
            active_task_id: Arc::new(RwLock::new(None)),
            window_visible: Arc::new(RwLock::new(false)),
            storage_path,
        }
    }

    fn load_tasks(path: &PathBuf) -> Vec<Task> {
        if let Ok(content) = std::fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn save_tasks(&self) {
        let tasks = self.tasks.read().clone();
        if let Some(parent) = self.storage_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&tasks) {
            let _ = std::fs::write(&self.storage_path, json);
        }
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.tasks.read().clone()
    }

    pub fn active_task(&self) -> Option<Task> {
        let task_id = *self.active_task_id.read();
        task_id.and_then(|id| {
            self.tasks.read().iter().find(|t| t.id == id).cloned()
        })
    }

    pub fn active_task_id(&self) -> Option<Uuid> {
        *self.active_task_id.read()
    }

    pub fn window_visible(&self) -> bool {
        *self.window_visible.read()
    }

    pub fn add_task(&mut self, task: Task, cx: &mut Context<Self>) {
        self.tasks.write().push(task);
        self.save_tasks();
        cx.emit(TaskStateChanged);
        cx.notify();
    }

    pub fn remove_task(&mut self, task_id: Uuid, cx: &mut Context<Self>) {
        self.tasks.write().retain(|t| t.id != task_id);
        
        if self.active_task_id.read().as_ref() == Some(&task_id) {
            *self.active_task_id.write() = None;
            cx.emit(TaskSelected { task_id: None });
        }
        
        self.save_tasks();
        cx.emit(TaskStateChanged);
        cx.notify();
    }

    pub fn toggle_task_completed(&mut self, task_id: Uuid, cx: &mut Context<Self>) {
        if let Some(task) = self.tasks.write().iter_mut().find(|t| t.id == task_id) {
            task.completed = !task.completed;
            self.save_tasks();
            cx.emit(TaskStateChanged);
            cx.notify();
        }
    }

    pub fn select_task(&mut self, task_id: Option<Uuid>, cx: &mut Context<Self>) {
        *self.active_task_id.write() = task_id;
        cx.emit(TaskSelected { task_id });
        cx.notify();
    }

    pub fn increment_active_task_pomodoros(&mut self, cx: &mut Context<Self>) {
        if let Some(task_id) = *self.active_task_id.read() {
            if let Some(task) = self.tasks.write().iter_mut().find(|t| t.id == task_id) {
                task.completed_pomodoros += 1;
                self.save_tasks();
                cx.emit(TaskStateChanged);
                cx.notify();
            }
        }
    }

    pub fn toggle_window(&mut self, cx: &mut Context<Self>) {
        let visible = !*self.window_visible.read();
        *self.window_visible.write() = visible;
        cx.emit(TaskWindowToggled);
        cx.notify();
    }
}

struct GlobalTaskService(Entity<TaskService>);
impl Global for GlobalTaskService {}

impl TaskService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalTaskService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalTaskService(service.clone()));
        service
    }
}
