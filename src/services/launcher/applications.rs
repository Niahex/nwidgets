use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use freedesktop_desktop_entry::{DesktopEntry, Iter};
use freedesktop_icons::lookup;

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
                if let Ok(entry) = DesktopEntry::decode(&path, &bytes) {
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

                    let raw_icon = entry.icon().map(|s| s.to_string());
                    let resolved_icon = if let Some(icon_name) = &raw_icon {
                        if Path::new(icon_name).exists() && Self::is_valid_svg(icon_name) {
                             Some(icon_name.clone())
                        } else if let Some(found_path) = lookup(icon_name).find() {
                            let path_str = found_path.to_string_lossy().to_string();
                            if Self::is_valid_svg(&path_str) {
                                Some(path_str)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let app = Application {
                        id: id.clone(),
                        name,
                        generic_name: entry.generic_name(None).map(|s| s.to_string()),
                        comment: entry.comment(None).map(|s| s.to_string()),
                        exec: entry.exec().unwrap_or("").to_string(),
                        icon: resolved_icon,
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

    fn is_valid_svg(path: &str) -> bool {
        if !path.ends_with(".svg") {
            return true;
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            !content.contains("xlink:href=\"data:image") && 
            !content.contains("<image")
        } else {
            false
        }
    }
}
