use crate::services::database::get_database;
use crate::services::system::HyprlandService;
use crate::widgets::r#macro::types::*;
use anyhow::Result;
use futures::channel::mpsc::unbounded;
use futures::StreamExt;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

pub struct MacroService {
    macros: Arc<RwLock<Vec<Macro>>>,
    recording: Arc<RwLock<bool>>,
    playing: Arc<RwLock<Option<Uuid>>>,
    current_recording: Arc<RwLock<Option<Macro>>>,
    playback_speed: Arc<RwLock<f32>>,
    visible: Arc<RwLock<bool>>,
}

impl EventEmitter<MacroToggled> for MacroService {}
impl EventEmitter<MacroRecordingChanged> for MacroService {}
impl EventEmitter<MacroPlayingChanged> for MacroService {}
impl EventEmitter<MacroListChanged> for MacroService {}

impl MacroService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let macros = Arc::new(RwLock::new(Vec::new()));
        let recording = Arc::new(RwLock::new(false));
        let playing = Arc::new(RwLock::new(None));
        let current_recording = Arc::new(RwLock::new(None));
        let playback_speed = Arc::new(RwLock::new(1.0));
        let visible = Arc::new(RwLock::new(false));

        let service = Self {
            macros: macros.clone(),
            recording,
            playing,
            current_recording,
            playback_speed,
            visible,
        };

        if let Ok(loaded) = Self::load_macros() {
            *macros.write() = loaded;
        }

        Self::start_dbus_server(cx, service.macros.clone());

        service
    }

    fn start_dbus_server(cx: &mut Context<Self>, macros: Arc<RwLock<Vec<Macro>>>) {
        static INIT: std::sync::Once = std::sync::Once::new();

        INIT.call_once(|| {
            log::info!("Starting macro D-Bus server");
            
            let (command_tx, mut command_rx) = unbounded();
            
            gpui_tokio::Tokio::spawn(cx, async move {
                if let Err(e) = crate::widgets::r#macro::service::run_dbus_server(command_tx, macros).await {
                    log::error!("Macro D-Bus error: {e}");
                }
            }).detach();
            
            cx.spawn(|this: WeakEntity<Self>, cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    while let Some(cmd) = command_rx.next().await {
                        use crate::widgets::r#macro::service::MacroDbusCommand;
                        
                        let _ = this.update(&mut cx, |service, cx| {
                            match cmd {
                                MacroDbusCommand::Toggle => service.toggle_window(cx),
                                MacroDbusCommand::StartRecording(name) => service.start_recording(name, cx),
                                MacroDbusCommand::StopRecording => service.stop_recording(cx),
                                MacroDbusCommand::PlayMacro(id) => service.play_macro(id, cx),
                                MacroDbusCommand::StopPlayback => service.stop_playback(cx),
                            }
                        });
                    }
                }
            })
            .detach();
        });
    }

    fn load_macros() -> Result<Vec<Macro>> {
        let db = get_database();
        let conn = db.conn();
        let conn = conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, name, app_class, created_at FROM macros ORDER BY created_at DESC"
        )?;

        let macros = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
            ))?;
            let name: String = row.get(1)?;
            let app_class: Option<String> = row.get(2)?;
            let created_at: u64 = row.get(3)?;

            let actions = Self::load_actions(&conn, &id_str)?;

            Ok(Macro {
                id,
                name,
                app_class,
                actions,
                created_at,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(macros)
    }

    fn load_actions(conn: &rusqlite::Connection, macro_id: &str) -> Result<Vec<MacroAction>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT timestamp_ms, action_type, action_data, click_zone_x, click_zone_y, 
                    click_zone_width, click_zone_height 
             FROM macro_actions 
             WHERE macro_id = ? 
             ORDER BY action_index"
        )?;

        let actions = stmt.query_map([macro_id], |row| {
            let timestamp_ms: u64 = row.get(0)?;
            let action_type_str: String = row.get(1)?;
            let action_data: Option<String> = row.get(2)?;
            
            let action_type = Self::parse_action_type(&action_type_str, action_data.as_deref())
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                ))?;

            let click_zone = if let (Some(x), Some(y), Some(w), Some(h)) = (
                row.get::<_, Option<i32>>(3)?,
                row.get::<_, Option<i32>>(4)?,
                row.get::<_, Option<u32>>(5)?,
                row.get::<_, Option<u32>>(6)?,
            ) {
                Some(ClickZone { x, y, width: w, height: h })
            } else {
                None
            };

            Ok(MacroAction {
                timestamp_ms,
                action_type,
                click_zone,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(actions)
    }

    fn parse_action_type(type_str: &str, data: Option<&str>) -> Result<ActionType> {
        match type_str {
            "KeyPress" => {
                let code = data.ok_or_else(|| anyhow::anyhow!("Missing data for KeyPress"))?
                    .parse::<u32>()?;
                Ok(ActionType::KeyPress(code))
            }
            "KeyRelease" => {
                let code = data.ok_or_else(|| anyhow::anyhow!("Missing data for KeyRelease"))?
                    .parse::<u32>()?;
                Ok(ActionType::KeyRelease(code))
            }
            "MouseClick" => {
                let btn_str = data.ok_or_else(|| anyhow::anyhow!("Missing data for MouseClick"))?;
                let btn = match btn_str {
                    "Left" => MacroMouseButton::Left,
                    "Right" => MacroMouseButton::Right,
                    "Middle" => MacroMouseButton::Middle,
                    _ => MacroMouseButton::Left,
                };
                Ok(ActionType::MouseClick(btn))
            }
            "Delay" => {
                let ms = data.ok_or_else(|| anyhow::anyhow!("Missing data for Delay"))?
                    .parse::<u64>()?;
                Ok(ActionType::Delay(ms))
            }
            _ => Err(anyhow::anyhow!("Unknown action type: {}", type_str)),
        }
    }

    fn save_macros(&self, cx: &mut Context<Self>) {
        let macros = self.macros.read().clone();
        
        gpui_tokio::Tokio::spawn(cx, async move {
            tokio::task::spawn_blocking(move || {
                if let Err(e) = Self::save_macros_sync(macros) {
                    log::error!("Failed to save macros: {}", e);
                }
            }).await
        }).detach();
    }

    fn save_macros_sync(macros: Vec<Macro>) -> Result<()> {
        let db = get_database();
        let conn = db.conn();
        let conn = conn.lock();

        conn.execute("DELETE FROM macro_actions", [])?;
        conn.execute("DELETE FROM macros", [])?;

        for macro_rec in macros {
            conn.execute(
                "INSERT INTO macros (id, name, app_class, created_at) VALUES (?, ?, ?, ?)",
                rusqlite::params![
                    macro_rec.id.to_string(),
                    macro_rec.name,
                    macro_rec.app_class,
                    macro_rec.created_at,
                ],
            )?;

            for (idx, action) in macro_rec.actions.iter().enumerate() {
                let (action_type_str, action_data) = Self::serialize_action_type(&action.action_type);
                
                conn.execute(
                    "INSERT INTO macro_actions 
                     (macro_id, action_index, timestamp_ms, action_type, action_data, 
                      click_zone_x, click_zone_y, click_zone_width, click_zone_height)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![
                        macro_rec.id.to_string(),
                        idx as i64,
                        action.timestamp_ms as i64,
                        action_type_str,
                        action_data,
                        action.click_zone.as_ref().map(|z| z.x),
                        action.click_zone.as_ref().map(|z| z.y),
                        action.click_zone.as_ref().map(|z| z.width as i64),
                        action.click_zone.as_ref().map(|z| z.height as i64),
                    ],
                )?;
            }
        }

        Ok(())
    }

    fn serialize_action_type(action_type: &ActionType) -> (&'static str, String) {
        match action_type {
            ActionType::KeyPress(code) => ("KeyPress", code.to_string()),
            ActionType::KeyRelease(code) => ("KeyRelease", code.to_string()),
            ActionType::MouseClick(btn) => {
                let btn_str = match btn {
                    MacroMouseButton::Left => "Left",
                    MacroMouseButton::Right => "Right",
                    MacroMouseButton::Middle => "Middle",
                };
                ("MouseClick", btn_str.to_string())
            }
            ActionType::Delay(ms) => ("Delay", ms.to_string()),
        }
    }

    pub fn toggle_window(&mut self, cx: &mut Context<Self>) {
        let mut visible = self.visible.write();
        *visible = !*visible;
        drop(visible);
        cx.emit(MacroToggled);
        cx.notify();
    }

    pub fn visible(&self) -> bool {
        *self.visible.read()
    }

    pub fn is_recording(&self) -> bool {
        *self.recording.read()
    }

    pub fn is_playing(&self) -> Option<Uuid> {
        *self.playing.read()
    }

    pub fn playback_speed(&self) -> f32 {
        *self.playback_speed.read()
    }

    pub fn set_playback_speed(&mut self, speed: f32, cx: &mut Context<Self>) {
        *self.playback_speed.write() = speed.clamp(0.1, 10.0);
        cx.notify();
    }

    pub fn get_macros(&self) -> Vec<Macro> {
        self.macros.read().clone()
    }

    pub fn start_recording(&mut self, name: String, cx: &mut Context<Self>) {
        if *self.recording.read() {
            return;
        }

        let hyprland = HyprlandService::global(cx);
        let app_class = hyprland
            .read(cx)
            .active_window()
            .map(|w| w.class.to_string());

        let new_macro = Macro::new(name, app_class);
        *self.current_recording.write() = Some(new_macro);
        *self.recording.write() = true;

        let recording = Arc::clone(&self.recording);
        let current_recording = Arc::clone(&self.current_recording);

        log::info!("Starting macro recording...");
        
        gpui_tokio::Tokio::spawn(cx, async move {
            log::info!("Recording task started");
            if let Err(e) = Self::record_events(recording, current_recording).await {
                log::error!("Recording error: {}", e);
            } else {
                log::info!("Recording task completed successfully");
            }
        }).detach();

        cx.emit(MacroRecordingChanged { recording: true });
        cx.notify();
    }

    pub fn stop_recording(&mut self, cx: &mut Context<Self>) {
        if !*self.recording.read() {
            return;
        }

        *self.recording.write() = false;

        if let Some(recorded_macro) = self.current_recording.write().take() {
            if !recorded_macro.actions.is_empty() {
                self.macros.write().push(recorded_macro);
                self.save_macros(cx);
                cx.emit(MacroListChanged);
            }
        }

        cx.emit(MacroRecordingChanged { recording: false });
        cx.notify();
    }

    async fn record_events(
        recording: Arc<RwLock<bool>>,
        current_recording: Arc<RwLock<Option<Macro>>>,
    ) -> Result<()> {
        log::info!("Getting input devices...");
        let devices = Self::get_input_devices()?;
        log::info!("Found {} input devices", devices.len());
        
        if devices.is_empty() {
            return Err(anyhow::anyhow!("No input devices accessible"));
        }

        let start_time = std::time::Instant::now();

        let mut handles = Vec::new();
        for device in devices {
            log::info!("Starting evtest for device: {}", device);
            let recording = Arc::clone(&recording);
            let current_recording = Arc::clone(&current_recording);
            let start_time = start_time;

            let handle = tokio::spawn(async move {
                let mut child = match Command::new("evtest")
                    .arg(&device)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(e) => {
                        log::error!("Failed to spawn evtest for {}: {}", device, e);
                        return;
                    }
                };

                let stdout = child.stdout.take().unwrap();
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while *recording.read() {
                    match tokio::time::timeout(
                        std::time::Duration::from_millis(100),
                        lines.next_line(),
                    )
                    .await
                    {
                        Ok(Ok(Some(line))) => {
                            if let Some(action) = Self::parse_event_line(&line) {
                                let timestamp_ms = start_time.elapsed().as_millis() as u64;
                                let mut recording = current_recording.write();
                                if let Some(ref mut macro_rec) = *recording {
                                    macro_rec.actions.push(MacroAction {
                                        timestamp_ms,
                                        action_type: action,
                                        click_zone: None,
                                    });
                                }
                            }
                        }
                        Ok(Ok(None)) => break,
                        Ok(Err(e)) => {
                            log::error!("Error reading evtest output: {}", e);
                            break;
                        }
                        Err(_) => continue,
                    }
                }

                let _ = child.kill().await;
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await;
        }

        Ok(())
    }

    fn get_input_devices() -> Result<Vec<String>> {
        let mut devices = Vec::new();
        let paths = ["/dev/input/by-id"];

        for base_path in paths {
            if let Ok(entries) = std::fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let path_str = path.to_string_lossy();
                    if (path_str.contains("event-kbd") || path_str.contains("event-mouse"))
                        && std::fs::metadata(&path).is_ok()
                    {
                        if let Ok(canonical) = std::fs::canonicalize(&path) {
                            devices.push(canonical.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        Ok(devices)
    }

    fn parse_event_line(line: &str) -> Option<ActionType> {
        if !line.contains("EV_KEY") || !line.contains("value") {
            return None;
        }

        let is_press = line.contains("value 1");
        let is_release = line.contains("value 0");

        if !is_press && !is_release {
            return None;
        }

        if line.contains("BTN_LEFT") {
            return Some(ActionType::MouseClick(MacroMouseButton::Left));
        } else if line.contains("BTN_RIGHT") {
            return Some(ActionType::MouseClick(MacroMouseButton::Right));
        } else if line.contains("BTN_MIDDLE") {
            return Some(ActionType::MouseClick(MacroMouseButton::Middle));
        } else if line.contains("KEY_") {
            if let Some(code_str) = line.split("code").nth(1) {
                if let Some(code) = code_str
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    return if is_press {
                        Some(ActionType::KeyPress(code))
                    } else {
                        Some(ActionType::KeyRelease(code))
                    };
                }
            }
        }

        None
    }

    pub fn play_macro(&mut self, macro_id: Uuid, cx: &mut Context<Self>) {
        if self.playing.read().is_some() {
            return;
        }

        let macro_to_play = self
            .macros
            .read()
            .iter()
            .find(|m| m.id == macro_id)
            .cloned();

        let Some(macro_to_play) = macro_to_play else {
            return;
        };

        *self.playing.write() = Some(macro_id);

        let playing = Arc::clone(&self.playing);
        let playback_speed = Arc::clone(&self.playback_speed);

        gpui_tokio::Tokio::spawn(cx, async move {
            if let Err(e) = Self::replay_macro(&macro_to_play, playing.clone(), playback_speed).await {
                log::error!("Playback error: {}", e);
            }
        }).detach();

        cx.emit(MacroPlayingChanged {
            playing: true,
            macro_id: Some(macro_id),
        });
        cx.notify();
    }

    pub fn stop_playback(&mut self, cx: &mut Context<Self>) {
        *self.playing.write() = None;
        cx.emit(MacroPlayingChanged {
            playing: false,
            macro_id: None,
        });
        cx.notify();
    }

    async fn replay_macro(
        macro_rec: &Macro,
        playing: Arc<RwLock<Option<Uuid>>>,
        playback_speed: Arc<RwLock<f32>>,
    ) -> Result<()> {
        let mut last_timestamp = 0u64;

        for action in &macro_rec.actions {
            if playing.read().is_none() {
                break;
            }

            let delay_ms = action.timestamp_ms.saturating_sub(last_timestamp);
            let speed = *playback_speed.read();
            let adjusted_delay = (delay_ms as f32 / speed) as u64;

            if adjusted_delay > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(adjusted_delay)).await;
            }

            match &action.action_type {
                ActionType::MouseClick(btn) => {
                    if let Some(zone) = &action.click_zone {
                        let (x, y) = Self::randomize_click_position(zone);
                        let _ = Command::new("ydotool")
                            .args(["mousemove", "--absolute", &x.to_string(), &y.to_string()])
                            .output()
                            .await;
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                    
                    let btn_code = match btn {
                        MacroMouseButton::Left => "0xC0",
                        MacroMouseButton::Right => "0xC1",
                        MacroMouseButton::Middle => "0xC2",
                    };
                    let _ = Command::new("ydotool")
                        .args(["click", btn_code])
                        .output()
                        .await;
                }
                ActionType::KeyPress(code) | ActionType::KeyRelease(code) => {
                    let _ = Command::new("ydotool")
                        .args(["key", &code.to_string()])
                        .output()
                        .await;
                }
                ActionType::Delay(_) => {
                    // Delay is already handled by the timestamp difference calculation above
                }
            }

            last_timestamp = action.timestamp_ms;
        }

        Ok(())
    }

    fn randomize_click_position(zone: &ClickZone) -> (i32, i32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let x = zone.x + rng.gen_range(0..zone.width as i32);
        let y = zone.y + rng.gen_range(0..zone.height as i32);
        (x, y)
    }

    pub fn delete_macro(&mut self, macro_id: Uuid, cx: &mut Context<Self>) {
        self.macros.write().retain(|m| m.id != macro_id);
        self.save_macros(cx);
        cx.emit(MacroListChanged);
        cx.notify();
    }

    pub fn rename_macro(&mut self, macro_id: Uuid, new_name: String, cx: &mut Context<Self>) {
        let renamed = {
            let mut macros = self.macros.write();
            if let Some(macro_rec) = macros.iter_mut().find(|m| m.id == macro_id) {
                macro_rec.name = new_name;
                true
            } else {
                false
            }
        };
        
        if renamed {
            self.save_macros(cx);
            cx.emit(MacroListChanged);
            cx.notify();
        }
    }

    pub fn add_action(&mut self, macro_id: Uuid, action: MacroAction, cx: &mut Context<Self>) {
        let added = {
            let mut macros = self.macros.write();
            if let Some(macro_rec) = macros.iter_mut().find(|m| m.id == macro_id) {
                macro_rec.actions.push(action);
                true
            } else {
                false
            }
        };
        
        if added {
            self.save_macros(cx);
            cx.emit(MacroListChanged);
            cx.notify();
        }
    }

    pub fn delete_action(&mut self, macro_id: Uuid, action_index: usize, cx: &mut Context<Self>) {
        log::info!("delete_action called: macro_id={}, action_index={}", macro_id, action_index);
        let deleted = {
            let mut macros = self.macros.write();
            if let Some(macro_rec) = macros.iter_mut().find(|m| m.id == macro_id) {
                log::info!("Found macro, actions count: {}", macro_rec.actions.len());
                if action_index < macro_rec.actions.len() {
                    log::info!("Removing action at index {}", action_index);
                    macro_rec.actions.remove(action_index);
                    log::info!("Action removed");
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        
        if deleted {
            log::info!("Saving macros...");
            self.save_macros(cx);
            log::info!("Emitting MacroListChanged...");
            cx.emit(MacroListChanged);
            log::info!("Calling notify...");
            cx.notify();
            log::info!("delete_action completed");
        }
    }

    pub fn move_action_up(&mut self, macro_id: Uuid, action_index: usize, cx: &mut Context<Self>) {
        let moved = {
            let mut macros = self.macros.write();
            if let Some(macro_rec) = macros.iter_mut().find(|m| m.id == macro_id) {
                if action_index > 0 && action_index < macro_rec.actions.len() {
                    macro_rec.actions.swap(action_index, action_index - 1);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        
        if moved {
            self.save_macros(cx);
            cx.emit(MacroListChanged);
            cx.notify();
        }
    }

    pub fn move_action_down(&mut self, macro_id: Uuid, action_index: usize, cx: &mut Context<Self>) {
        let moved = {
            let mut macros = self.macros.write();
            if let Some(macro_rec) = macros.iter_mut().find(|m| m.id == macro_id) {
                if action_index < macro_rec.actions.len().saturating_sub(1) {
                    macro_rec.actions.swap(action_index, action_index + 1);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        
        if moved {
            self.save_macros(cx);
            cx.emit(MacroListChanged);
            cx.notify();
        }
    }

    pub fn update_action(&mut self, macro_id: Uuid, action_index: usize, action: MacroAction, cx: &mut Context<Self>) {
        let updated = {
            let mut macros = self.macros.write();
            if let Some(macro_rec) = macros.iter_mut().find(|m| m.id == macro_id) {
                if action_index < macro_rec.actions.len() {
                    macro_rec.actions[action_index] = action;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        
        if updated {
            self.save_macros(cx);
            cx.emit(MacroListChanged);
            cx.notify();
        }
    }
}

struct GlobalMacroService(Entity<MacroService>);
impl Global for GlobalMacroService {}

impl MacroService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalMacroService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalMacroService(service.clone()));
        service
    }
}
