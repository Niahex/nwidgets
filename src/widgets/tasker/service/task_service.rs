use crate::services::database::get_database;
use crate::widgets::tasker::types::{Task, TaskSelected, TaskStateChanged, TaskWindowToggled};
use anyhow::Result;
use chrono::{DateTime, Utc};
use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

pub struct TaskService {
    tasks: Arc<RwLock<Vec<Task>>>,
    active_task_id: Arc<RwLock<Option<Uuid>>>,
    window_visible: Arc<RwLock<bool>>,
}

impl EventEmitter<TaskStateChanged> for TaskService {}
impl EventEmitter<TaskSelected> for TaskService {}
impl EventEmitter<TaskWindowToggled> for TaskService {}

impl TaskService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let tasks = Self::load_tasks().unwrap_or_default();

        Self {
            tasks: Arc::new(RwLock::new(tasks)),
            active_task_id: Arc::new(RwLock::new(None)),
            window_visible: Arc::new(RwLock::new(false)),
        }
    }

    fn load_tasks() -> Result<Vec<Task>> {
        let db = get_database()?;
        let conn = db.conn();
        let conn = conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, title, project, time_spent_secs, created_at, completed 
             FROM tasks 
             ORDER BY created_at DESC",
        )?;

        let tasks = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let id = Uuid::parse_str(&id_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e.to_string(),
                        )),
                    )
                })?;
                let title: String = row.get(1)?;
                let project: Option<String> = row.get(2)?;
                let time_spent_secs: u64 = row.get(3)?;
                let created_at_timestamp: i64 = row.get(4)?;
                let completed: bool = row.get(5)?;

                let created_at =
                    DateTime::from_timestamp(created_at_timestamp, 0).unwrap_or_else(|| Utc::now());

                Ok(Task {
                    id,
                    title,
                    project,
                    time_spent_secs,
                    created_at,
                    completed,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    fn save_tasks(&self, cx: &mut Context<Self>) {
        let tasks = self.tasks.read().clone();
        
        gpui_tokio::Tokio::spawn(cx, async move {
            tokio::task::spawn_blocking(move || {
                if let Err(e) = Self::save_tasks_sync(tasks) {
                    log::error!("Failed to save tasks: {}", e);
                }
            }).await
        }).detach();
    }

    fn save_tasks_sync(tasks: Vec<Task>) -> Result<()> {
        let db = get_database()?;
        let conn = db.conn();
        let conn = conn.lock();

        conn.execute("DELETE FROM tasks", [])?;

        for task in tasks {
            conn.execute(
                "INSERT INTO tasks 
                 (id, title, project, time_spent_secs, created_at, completed)
                 VALUES (?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    task.id.to_string(),
                    task.title,
                    task.project,
                    task.time_spent_secs as i64,
                    task.created_at.timestamp(),
                    task.completed,
                ],
            )?;
        }

        Ok(())
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.tasks.read().clone()
    }

    pub fn active_task(&self) -> Option<Task> {
        let task_id = *self.active_task_id.read();
        task_id.and_then(|id| self.tasks.read().iter().find(|t| t.id == id).cloned())
    }

    pub fn active_task_id(&self) -> Option<Uuid> {
        *self.active_task_id.read()
    }

    pub fn window_visible(&self) -> bool {
        *self.window_visible.read()
    }

    pub fn add_task(&mut self, task: Task, cx: &mut Context<Self>) {
        self.tasks.write().push(task);
        self.save_tasks(cx);
        cx.emit(TaskStateChanged);
        cx.notify();
    }

    pub fn remove_task(&mut self, task_id: Uuid, cx: &mut Context<Self>) {
        self.tasks.write().retain(|t| t.id != task_id);

        if self.active_task_id.read().as_ref() == Some(&task_id) {
            *self.active_task_id.write() = None;
            cx.emit(TaskSelected { task_id: None });
        }

        self.save_tasks(cx);
        cx.emit(TaskStateChanged);
        cx.notify();
    }

    pub fn toggle_task_completed(&mut self, task_id: Uuid, cx: &mut Context<Self>) {
        let found = {
            let mut tasks = self.tasks.write();
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                task.completed = !task.completed;
                true
            } else {
                false
            }
        };
        if found {
            self.save_tasks(cx);
            cx.emit(TaskStateChanged);
            cx.notify();
        }
    }

    pub fn select_task(&mut self, task_id: Option<Uuid>, cx: &mut Context<Self>) {
        *self.active_task_id.write() = task_id;
        cx.emit(TaskSelected { task_id });
        cx.notify();
    }

    pub fn add_time_spent_to_active_task(&mut self, secs: u64, cx: &mut Context<Self>) {
        let should_save = if let Some(task_id) = *self.active_task_id.read() {
            let mut tasks = self.tasks.write();
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                task.time_spent_secs += secs;
                task.time_spent_secs % 60 == 0
            } else {
                false
            }
        } else {
            false
        };
        if should_save {
            self.save_tasks(cx);
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
