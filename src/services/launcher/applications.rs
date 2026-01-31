use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use freedesktop_desktop_entry::{DesktopEntry, Iter};

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub desktop_file: PathBuf,
}

#[derive(Clone)]
pub struct ApplicationService {
    state: Arc<RwLock<ApplicationState>>,
}

#[derive(Default)]
struct ApplicationState {
    applications: HashMap<String, Application>,
    sorted_apps: Vec<String>,
}

impl ApplicationService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(ApplicationState::default())),
        };

        service.scan_applications();
        service
    }

    fn scan_applications(&self) {
        let state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            Self::do_scan(&state);
        });
    }

    fn do_scan(state: &Arc<RwLock<ApplicationState>>) {
        let mut apps = HashMap::new();

        let locales = freedesktop_desktop_entry::default_paths();

        for path in Iter::new(locales) {
            if let Ok(bytes) = std::fs::read_to_string(&path) {
                if let Ok(entry) = DesktopEntry::from_str(&path, &bytes, None::<&[&str]>) {
                    if entry.no_display() {
                        continue;
                    }

                    let name = entry.name(None).map(|s| s.to_string()).unwrap_or_default();
                    if name.is_empty() {
                        continue;
                    }

                    let id = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();

                    let app = Application {
                        id: id.clone(),
                        name,
                        generic_name: entry.generic_name(None).map(|s| s.to_string()),
                        comment: entry.comment(None).map(|s| s.to_string()),
                        exec: entry.exec().unwrap_or("").to_string(),
                        icon: entry.icon().map(|s| s.to_string()),
                        categories: entry.categories()
                            .map(|s| s.split(';').map(|s| s.to_string()).collect())
                            .unwrap_or_default(),
                        keywords: vec![],
                        desktop_file: path.clone(),
                    };

                    apps.insert(id, app);
                }
            }
        }

        let mut sorted: Vec<String> = apps.keys().cloned().collect();
        sorted.sort_by(|a, b| {
            let app_a = apps.get(a).map(|a| &a.name).unwrap_or(a);
            let app_b = apps.get(b).map(|a| &a.name).unwrap_or(b);
            app_a.to_lowercase().cmp(&app_b.to_lowercase())
        });

        let mut s = state.write();
        s.applications = apps;
        s.sorted_apps = sorted;

        log::info!("Scanned {} applications", s.applications.len());
    }

    pub fn get_all(&self) -> Vec<Application> {
        let state = self.state.read();
        state.sorted_apps.iter()
            .filter_map(|id| state.applications.get(id))
            .cloned()
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<Application> {
        self.state.read().applications.get(id).cloned()
    }

    pub fn launch(&self, id: &str) -> anyhow::Result<()> {
        let app = self.get(id).ok_or(anyhow::anyhow!("App not found"))?;

        let exec = app.exec
            .replace("%f", "")
            .replace("%F", "")
            .replace("%u", "")
            .replace("%U", "")
            .replace("%c", &app.name)
            .trim()
            .to_string();

        std::process::Command::new("sh")
            .arg("-c")
            .arg(&exec)
            .spawn()?;

        Ok(())
    }
}
