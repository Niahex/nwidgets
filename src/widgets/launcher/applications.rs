use crate::widgets::launcher::state::ApplicationInfo;
use freedesktop_desktop_entry::{DesktopEntry, Iter};
use freedesktop_icons::lookup;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn get_cache_path() -> String {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        format!("{runtime_dir}/nlauncher_cache.json")
    } else {
        "/tmp/nlauncher_cache.json".to_string()
    }
}

pub fn load_from_cache() -> Option<Vec<ApplicationInfo>> {
    let cache_path = get_cache_path();
    let cache_path = Path::new(&cache_path);

    if !cache_path.exists() {
        return None;
    }

    // Accept even slightly old cache for initial speed
    let content = fs::read_to_string(cache_path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_to_cache(applications: &[ApplicationInfo]) -> Result<(), Box<dyn std::error::Error>> {
    let cache_path = get_cache_path();
    let content = serde_json::to_string(applications)?;
    fs::write(cache_path, content)?;
    Ok(())
}

pub fn scan_applications() -> Vec<ApplicationInfo> {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let applications = Arc::new(Mutex::new(Vec::new()));
    let seen_names = Arc::new(Mutex::new(HashSet::new()));

    let paths: Vec<_> = Iter::new(freedesktop_desktop_entry::default_paths()).collect();
    let chunk_size = (paths.len() / 4).max(1);

    let mut handles = Vec::new();

    for chunk in paths.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let applications = Arc::clone(&applications);
        let seen_names = Arc::clone(&seen_names);

        let handle = thread::spawn(move || {
            let mut local_apps = Vec::new();

            for path in chunk {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(desktop_entry) = DesktopEntry::decode(&path, &content) {
                        if let Some(name) = desktop_entry.name(None) {
                            if let Some(exec) = desktop_entry.exec() {
                                {
                                    let mut seen = seen_names.lock().unwrap();
                                    if seen.contains(&name.to_string()) {
                                        continue;
                                    }
                                    seen.insert(name.to_string());
                                }

                                let icon_path = desktop_entry
                                    .icon()
                                    .and_then(|icon_name| lookup(icon_name).with_size(24).find())
                                    .map(|p| p.to_string_lossy().to_string());

                                let exec_clean = exec
                                    .split_whitespace()
                                    .filter(|part| !part.starts_with('%'))
                                    .collect::<Vec<_>>()
                                    .join(" ");

                                local_apps.push(ApplicationInfo {
                                    name: name.to_string(),
                                    name_lower: name.to_lowercase(),
                                    exec: exec_clean,
                                    icon: desktop_entry.icon().map(|s| s.to_string()),
                                    icon_path,
                                });
                            }
                        }
                    }
                }
            }

            let mut apps = applications.lock().unwrap();
            apps.extend(local_apps);
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    let mut applications = Arc::try_unwrap(applications).unwrap().into_inner().unwrap();
    applications.sort_by(|a, b| a.name.cmp(&b.name));
    applications
}
