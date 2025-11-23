use gtk4::{gio, prelude::*};
use glib::MainContext;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use walkdir::WalkDir;
use std::fs::{self, File};
use std::io::{Read, Write};
use serde::{de::DeserializeOwned, Serialize};

pub struct ApplicationsService;

impl ApplicationsService {
    /// Trouve tous les fichiers .desktop dans les répertoires XDG
    fn find_desktop_files() -> Vec<PathBuf> {
        let mut desktop_files = HashSet::new();
        let mut data_dirs = Vec::new();

        if let Ok(xdg_data_dirs) = env::var("XDG_DATA_DIRS") {
            println!("[APPLICATIONS] Using XDG_DATA_DIRS: {}", xdg_data_dirs);
            data_dirs.extend(xdg_data_dirs.split(':').map(String::from));
        } else {
            println!("[APPLICATIONS] XDG_DATA_DIRS not set. Falling back to default paths.");
            if let Some(home_dir) = dirs::home_dir() {
                if let Some(local_share) = home_dir.join(".local/share").to_str() {
                    data_dirs.push(local_share.to_string());
                }
            }
            data_dirs.push("/usr/share".to_string());
            data_dirs.push("/usr/local/share".to_string());
            data_dirs.push("/run/current-system/sw/share".to_string());
        }

        for data_dir in data_dirs {
            let path = PathBuf::from(data_dir);
            let app_dir = if path.file_name().and_then(|s| s.to_str()) == Some("applications") {
                path
            } else {
                path.join("applications")
            };

            if app_dir.is_dir() {
                println!("[APPLICATIONS] Scanning for applications in: {:?}", app_dir);
                for entry in WalkDir::new(&app_dir)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("desktop"))
                {
                    desktop_files.insert(entry.path().to_path_buf());
                }
            }
        }

        let mut sorted_files: Vec<_> = desktop_files.into_iter().collect();
        sorted_files.sort();
        sorted_files
    }

    /// Charge les applications depuis le cache
    pub fn get_cached_applications() -> gio::ListStore {
        let app_list_store = gio::ListStore::new::<gio::AppInfo>();
        if let Some(app_ids) = Self::load_from_cache::<Vec<String>>() {
            println!("[APPLICATIONS] Loading {} applications from cache.", app_ids.len());
            for id in app_ids {
                if let Some(desktop_app_info) = gio::DesktopAppInfo::new(&id) {
                    let app_info = desktop_app_info.upcast::<gio::AppInfo>();
                    app_list_store.append(&app_info);
                }
            }
        }
        app_list_store
    }

    /// Scan complet du système pour les applications
    pub fn scan_for_applications() -> Vec<String> {
        println!("[APPLICATIONS] Scanning system for applications.");
        let mut app_ids = HashSet::new();

        // Méthode 1: Scan GIO standard
        println!("[APPLICATIONS] Scanning with gio::AppInfo::all()");
        let all_apps = gio::AppInfo::all();
        for app_info in all_apps {
            if app_info.should_show() {
                if let Some(id) = app_info.id() {
                    app_ids.insert(id.to_string());
                }
            }
        }

        // Méthode 2: Scan manuel des fichiers .desktop
        println!("[APPLICATIONS] Performing manual scan of XDG_DATA_DIRS.");
        let desktop_files = Self::find_desktop_files();
        for file_path in desktop_files {
            if let Some(app_info) = gio::DesktopAppInfo::from_filename(&file_path) {
                let app = app_info.upcast::<gio::AppInfo>();
                if app.should_show() {
                    if let Some(id) = app.id() {
                        app_ids.insert(id.to_string());
                    }
                }
            } else {
                println!("[APPLICATIONS] Could not create AppInfo from file: {:?}", file_path);
            }
        }

        let mut sorted_app_ids: Vec<_> = app_ids.into_iter().collect();
        sorted_app_ids.sort();
        println!("[APPLICATIONS] Found {} applications total.", sorted_app_ids.len());
        sorted_app_ids
    }

    /// Retourne le chemin du répertoire cache
    fn get_cache_dir() -> Result<PathBuf, std::io::Error> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Cache directory not found"))?
            .join("nwidgets");

        fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir)
    }

    /// Retourne le chemin complet du fichier cache
    fn get_cache_file_path() -> Result<PathBuf, std::io::Error> {
        Self::get_cache_dir().map(|dir| dir.join("app_ids.json"))
    }

    /// Charge les données depuis le cache JSON
    fn load_from_cache<T: DeserializeOwned>() -> Option<T> {
        let path = Self::get_cache_file_path().ok()?;
        let mut file = File::open(path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Sauvegarde les données dans le cache JSON
    pub fn save_to_cache<T: Serialize>(data: &T) -> Result<(), std::io::Error> {
        let path = Self::get_cache_file_path()?;
        let json_data = serde_json::to_string_pretty(data)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())
    }

    /// Efface le cache
    pub fn clear_cache() -> Result<(), std::io::Error> {
        if let Ok(path) = Self::get_cache_file_path() {
            if path.exists() {
                println!("[APPLICATIONS] Clearing cache file at: {:?}", path);
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    /// Lance le monitoring des applications en arrière-plan
    /// Le callback est appelé avec la liste des app_ids quand un scan est terminé
    pub fn start_monitoring<F>(callback: F)
    where
        F: Fn(Vec<String>) + 'static,
    {
        println!("[APPLICATIONS] Starting background monitoring");

        let (tx, rx) = mpsc::channel();

        // Thread de scan en arrière-plan
        thread::spawn(move || {
            println!("[APPLICATIONS] Background scan thread started");

            // Premier scan au démarrage
            let app_ids = Self::scan_for_applications();

            // Sauvegarder dans le cache
            if let Err(e) = Self::save_to_cache(&app_ids) {
                eprintln!("[APPLICATIONS] Failed to save cache: {}", e);
            }

            // Envoyer au thread GTK
            if tx.send(app_ids).is_err() {
                eprintln!("[APPLICATIONS] Failed to send app_ids");
            }

            println!("[APPLICATIONS] Background scan complete");
        });

        // Créer un async channel pour exécuter le callback sur le thread principal
        let (async_tx, async_rx) = async_channel::unbounded();

        // Thread qui reçoit les mises à jour et les transfère au async channel
        thread::spawn(move || {
            while let Ok(app_ids) = rx.recv() {
                if async_tx.send_blocking(app_ids).is_err() {
                    break;
                }
            }
        });

        // Attacher le callback au async channel
        MainContext::default().spawn_local(async move {
            while let Ok(app_ids) = async_rx.recv().await {
                println!("[APPLICATIONS] Received {} app IDs from background scan", app_ids.len());
                callback(app_ids);
            }
        });
    }
}

