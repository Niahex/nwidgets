use anyhow::Result;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_schema()?;
        Ok(db)
    }

    fn get_db_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("nwidgets")
            .join("nwidgets.db")
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS macros (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                app_class TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS macro_actions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                macro_id TEXT NOT NULL,
                action_index INTEGER NOT NULL,
                timestamp_ms INTEGER NOT NULL,
                action_type TEXT NOT NULL,
                action_data TEXT,
                click_zone_x INTEGER,
                click_zone_y INTEGER,
                click_zone_width INTEGER,
                click_zone_height INTEGER,
                FOREIGN KEY (macro_id) REFERENCES macros(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_macro_actions_macro_id 
             ON macro_actions(macro_id)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                project TEXT,
                time_spent_secs INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'Todo',
                priority INTEGER NOT NULL DEFAULT 5,
                due_date TEXT
            )",
            [],
        )?;

        self.migrate_tasks_table(&conn)?;

        Ok(())
    }

    fn migrate_tasks_table(&self, conn: &Connection) -> Result<()> {
        let has_status: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('tasks') WHERE name='status'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        let has_priority: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('tasks') WHERE name='priority'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        let has_due_date: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('tasks') WHERE name='due_date'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            > 0;

        if !has_status {
            conn.execute(
                "ALTER TABLE tasks ADD COLUMN status TEXT NOT NULL DEFAULT 'Todo'",
                [],
            )?;
        }

        if !has_priority {
            conn.execute(
                "ALTER TABLE tasks ADD COLUMN priority INTEGER NOT NULL DEFAULT 5",
                [],
            )?;
        }

        if !has_due_date {
            conn.execute("ALTER TABLE tasks ADD COLUMN due_date TEXT", [])?;
        }

        Ok(())
    }

    pub fn conn(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }
}

static DB: once_cell::sync::OnceCell<Database> = once_cell::sync::OnceCell::new();

pub fn init_database() -> Result<()> {
    DB.get_or_try_init(|| Database::new())?;
    Ok(())
}

pub fn get_database() -> Result<&'static Database> {
    DB.get()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized. Call init_database() first."))
}
