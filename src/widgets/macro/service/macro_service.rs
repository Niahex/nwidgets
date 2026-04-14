use crate::services::system::HyprlandService;
use crate::widgets::r#macro::types::*;
use anyhow::Result;
use futures::channel::mpsc::unbounded;
use futures::StreamExt;
use gpui::*;
use parking_lot::RwLock;
use std::process::Stdio;
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
        super::database::load_macros()
    }

    fn load_actions(conn: &rusqlite::Connection, macro_id: &str) -> Result<Vec<MacroAction>, rusqlite::Error> {
        super::database::load_actions(conn, macro_id)
    }

    fn parse_action_type(type_str: &str, data: Option<&str>) -> Result<ActionType> {
        super::database::parse_action_type(type_str, data)
    }

    fn save_macros(&self, cx: &mut Context<Self>) {
        let macros = self.macros.read().clone();
        
        gpui_tokio::Tokio::spawn(cx, async move {
            tokio::task::spawn_blocking(move || {
                if let Err(e) = super::database::save_macros_sync(macros) {
                    log::error!("Failed to save macros: {}", e);
                }
            }).await
        }).detach();
    }

    fn save_macros_sync(macros: Vec<Macro>) -> Result<()> {
        super::database::save_macros_sync(macros)
    }

    fn serialize_action_type(action_type: &ActionType) -> (&'static str, String) {
        super::database::serialize_action_type(action_type)
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
                let device_name = device.clone();
                let mut child = match Command::new("evtest")
                    .arg(device)
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(e) => {
                        log::error!("Failed to spawn evtest for {}: {}", device_name, e);
                        return;
                    }
                };

                let stdout = match child.stdout.take() {
                    Some(stdout) => stdout,
                    None => {
                        log::error!("Failed to capture stdout for evtest on {}", device_name);
                        return;
                    }
                };
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
        super::recording::get_input_devices()
    }

    fn parse_event_line(line: &str) -> Option<ActionType> {
        super::recording::parse_event_line(line)
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
            if let Err(e) = super::playback::replay_macro(&macro_to_play, playing.clone(), playback_speed).await {
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
