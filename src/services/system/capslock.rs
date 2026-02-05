use parking_lot::RwLock;
use std::sync::Arc;
use std::path::PathBuf;

use crate::TOKIO_RUNTIME;

pub struct CapsLockService {
    state: Arc<RwLock<bool>>,
    led_path: Option<PathBuf>,
}

impl CapsLockService {
    pub fn new() -> Self {
        let state = Arc::new(RwLock::new(false));
        
        let led_path = Self::find_capslock_led();
        
        if let Some(path) = &led_path {
            let state_clone = state.clone();
            let path_clone = path.clone();
            
            TOKIO_RUNTIME.spawn(async move {
                Self::monitor_capslock(state_clone, path_clone).await;
            });
        } else {
            ::log::warn!("CapsLock LED not found in /sys/class/leds/");
        }
        
        Self {
            state,
            led_path,
        }
    }
    
    fn find_capslock_led() -> Option<PathBuf> {
        let leds_dir = std::path::Path::new("/sys/class/leds");
        
        if let Ok(entries) = std::fs::read_dir(leds_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                if name_str.contains("capslock") {
                    let brightness_path = entry.path().join("brightness");
                    if brightness_path.exists() {
                        return Some(brightness_path);
                    }
                }
            }
        }
        
        None
    }
    
    async fn monitor_capslock(state: Arc<RwLock<bool>>, led_path: PathBuf) {
        loop {
            if let Ok(content) = tokio::fs::read_to_string(&led_path).await {
                let is_on = content.trim() == "1";
                let mut current_state = state.write();
                
                if *current_state != is_on {
                    *current_state = is_on;
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        *self.state.read()
    }
    
    pub fn on_change<F>(&self, callback: F)
    where
        F: Fn(bool) + Send + 'static,
    {
        let state = self.state.clone();
        let led_path = self.led_path.clone();
        
        if let Some(path) = led_path {
            TOKIO_RUNTIME.spawn(async move {
                let mut last_state = *state.read();
                
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    
                    let current_state = *state.read();
                    if current_state != last_state {
                        callback(current_state);
                        last_state = current_state;
                    }
                }
            });
        }
    }
}
