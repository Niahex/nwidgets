use gpui::prelude::*;
use gpui::{App, Context, Entity, EventEmitter, Global};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pipewire as pw;
use futures::StreamExt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioState {
    pub sink_volume: u8,
    pub sink_muted: bool,
    pub source_volume: u8,
    pub source_muted: bool,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            sink_volume: 50,
            sink_muted: false,
            source_volume: 50,
            source_muted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioDevice {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioStream {
    pub id: u32,
    pub app_name: String,
    pub window_title: Option<String>,
    pub volume: u8,
    pub muted: bool,
}

#[derive(Clone)]
pub struct AudioStateChanged {
    pub state: AudioState,
}

pub struct AudioService {
    state: Arc<RwLock<AudioState>>,
    sinks: Arc<RwLock<Vec<AudioDevice>>>,
    sources: Arc<RwLock<Vec<AudioDevice>>>,
    sink_inputs: Arc<RwLock<Vec<AudioStream>>>,
    source_outputs: Arc<RwLock<Vec<AudioStream>>>,
}

impl EventEmitter<AudioStateChanged> for AudioService {}

impl AudioService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let initial_state = Self::get_initial_state();
        let state = Arc::new(RwLock::new(initial_state));
        let sinks = Arc::new(RwLock::new(Self::get_sinks_wpctl()));
        let sources = Arc::new(RwLock::new(Self::get_sources_wpctl()));
        let sink_inputs = Arc::new(RwLock::new(Self::get_sink_inputs_wpctl()));
        let source_outputs = Arc::new(RwLock::new(Self::get_source_outputs_wpctl()));
        
        let state_clone = Arc::clone(&state);
        let sinks_clone = Arc::clone(&sinks);
        let sources_clone = Arc::clone(&sources);
        let sink_inputs_clone = Arc::clone(&sink_inputs);
        let source_outputs_clone = Arc::clone(&source_outputs);
        
        cx.spawn(async move |this, cx| {
            Self::monitor_pipewire(this, state_clone, sinks_clone, sources_clone, sink_inputs_clone, source_outputs_clone, cx).await
        }).detach();

        Self { state, sinks, sources, sink_inputs, source_outputs }
    }

    fn get_initial_state() -> AudioState {
        AudioState {
            sink_volume: Self::get_volume_wpctl("@DEFAULT_AUDIO_SINK@"),
            sink_muted: false,
            source_volume: Self::get_volume_wpctl("@DEFAULT_AUDIO_SOURCE@"),
            source_muted: false,
        }
    }

    fn get_volume_wpctl(device: &str) -> u8 {
        if let Ok(output) = std::process::Command::new("wpctl")
            .args(["get-volume", device])
            .output()
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                if let Some(vol_str) = text.split_whitespace().nth(1) {
                    if let Ok(vol) = vol_str.parse::<f32>() {
                        return (vol * 100.0).round() as u8;
                    }
                }
            }
        }
        50
    }

    fn get_sinks_wpctl() -> Vec<AudioDevice> {
        Self::parse_wpctl_status("Audio/Sink")
    }

    fn get_sources_wpctl() -> Vec<AudioDevice> {
        Self::parse_wpctl_status("Audio/Source")
    }

    fn parse_wpctl_status(device_type: &str) -> Vec<AudioDevice> {
        let output = std::process::Command::new("wpctl")
            .args(["status"])
            .output()
            .ok();
        
        let Some(output) = output else { return vec![] };
        let Ok(text) = String::from_utf8(output.stdout) else { return vec![] };
        
        let mut devices = vec![];
        let mut in_section = false;
        let section_marker = if device_type == "Audio/Sink" { "Sinks:" } else { "Sources:" };
        
        for line in text.lines() {
            if line.contains(section_marker) {
                in_section = true;
                continue;
            }
            if in_section && (line.contains("Sink endpoints:") || line.contains("Source endpoints:") || line.contains("Streams:") || (line.starts_with(" ") && line.trim().is_empty())) {
                break;
            }
            if in_section && line.contains(". ") {
                let is_default = line.contains("*");
                let line = line.replace("*", "").trim().to_string();
                if let Some((id_part, rest)) = line.split_once(". ") {
                    if let Ok(id) = id_part.trim().parse::<u32>() {
                        let name = rest.trim().to_string();
                        let description = name.split('[').next().unwrap_or(&name).trim().to_string();
                        devices.push(AudioDevice { id, name: name.clone(), description, is_default });
                    }
                }
            }
        }
        devices
    }

    fn get_sink_inputs_wpctl() -> Vec<AudioStream> {
        Self::parse_wpctl_streams(true)
    }

    fn get_source_outputs_wpctl() -> Vec<AudioStream> {
        Self::parse_wpctl_streams(false)
    }

    fn parse_wpctl_streams(is_playback: bool) -> Vec<AudioStream> {
        let output = std::process::Command::new("wpctl")
            .args(["status"])
            .output()
            .ok();
        
        let Some(output) = output else { return vec![] };
        let Ok(text) = String::from_utf8(output.stdout) else { return vec![] };
        
        let mut streams = vec![];
        let mut in_section = false;
        let section_marker = if is_playback { "Streams:" } else { "Capture:" };
        
        for line in text.lines() {
            if line.contains(section_marker) {
                in_section = true;
                continue;
            }
            if in_section && !line.starts_with(" ") && !line.trim().is_empty() {
                break;
            }
            if in_section && line.contains(". ") {
                let line = line.replace("*", "").trim().to_string();
                if let Some((id_part, rest)) = line.split_once(". ") {
                    if let Ok(id) = id_part.trim().parse::<u32>() {
                        let app_name = rest.trim().to_string();
                        streams.push(AudioStream {
                            id,
                            app_name,
                            window_title: None,
                            volume: 100,
                            muted: false,
                        });
                    }
                }
            }
        }
        streams
    }

    async fn monitor_pipewire(
        this: gpui::WeakEntity<Self>,
        state: Arc<RwLock<AudioState>>,
        sinks: Arc<RwLock<Vec<AudioDevice>>>,
        sources: Arc<RwLock<Vec<AudioDevice>>>,
        sink_inputs: Arc<RwLock<Vec<AudioStream>>>,
        source_outputs: Arc<RwLock<Vec<AudioStream>>>,
        cx: &mut gpui::AsyncApp,
    ) {
        loop {
            let (tx, mut rx) = futures::channel::mpsc::unbounded::<()>();
            
            // Spawn PipeWire listener in background thread
            std::thread::spawn({
                let tx = tx;
                move || {
                    pw::init();
                    
                    let mainloop = match pw::main_loop::MainLoopRc::new(None) {
                        Ok(ml) => ml,
                        Err(e) => {
                            eprintln!("[AudioService] Failed to create PipeWire mainloop: {e}");
                            return;
                        }
                    };
                    
                    let context = match pw::context::ContextRc::new(&mainloop, None) {
                        Ok(ctx) => ctx,
                        Err(e) => {
                            eprintln!("[AudioService] Failed to create PipeWire context: {e}");
                            return;
                        }
                    };
                    
                    let core = match context.connect_rc(None) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("[AudioService] Failed to connect to PipeWire: {e}");
                            return;
                        }
                    };
                    
                    let registry = match core.get_registry_rc() {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("[AudioService] Failed to get PipeWire registry: {e}");
                            return;
                        }
                    };
                    
                    // Store nodes to keep them alive
                    let nodes: std::rc::Rc<std::cell::RefCell<Vec<pw::node::Node>>> = 
                        std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
                    let listeners: std::rc::Rc<std::cell::RefCell<Vec<pw::node::NodeListener>>> = 
                        std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
                    
                    let nodes_clone = nodes.clone();
                    let listeners_clone = listeners.clone();
                    let registry_clone = registry.clone();
                    
                    let _registry_listener = registry
                        .add_listener_local()
                        .global(move |global| {
                            if global.type_ == pw::types::ObjectType::Node {
                                if let Some(props) = &global.props {
                                    let media_class = props.get("media.class");
                                    
                                    if let Some(class) = media_class {
                                        if class.contains("Audio/Sink") || class.contains("Audio/Source") {
                                            if let Ok(node) = registry_clone.bind::<pw::node::Node, _>(global) {
                                                let tx_param = tx.clone();
                                                
                                                let node_listener = node
                                                    .add_listener_local()
                                                    .param(move |_, _, _, _, _| {
                                                        let _ = tx_param.unbounded_send(());
                                                    })
                                                    .register();
                                                
                                                node.subscribe_params(&[pw::spa::param::ParamType::Props]);
                                                
                                                nodes_clone.borrow_mut().push(node);
                                                listeners_clone.borrow_mut().push(node_listener);
                                            }
                                        }
                                    }
                                }
                            }
                        })
                        .register();
                    
                    mainloop.run();
                }
            });
            
            // Process events with debouncing
            let mut last_update = std::time::Instant::now();
            let debounce_duration = std::time::Duration::from_millis(50);
            
            while let Some(()) = rx.next().await {
                let now = std::time::Instant::now();
                if now.duration_since(last_update) < debounce_duration {
                    while rx.try_next().is_ok() {}
                    cx.background_executor()
                        .timer(debounce_duration - now.duration_since(last_update))
                        .await;
                }
                
                last_update = std::time::Instant::now();
                
                let new_state = Self::get_initial_state();
                *sinks.write() = Self::get_sinks_wpctl();
                *sources.write() = Self::get_sources_wpctl();
                *sink_inputs.write() = Self::get_sink_inputs_wpctl();
                *source_outputs.write() = Self::get_source_outputs_wpctl();
                
                let changed = {
                    let mut current = state.write();
                    let changed = *current != new_state;
                    if changed {
                        *current = new_state.clone();
                    }
                    changed
                };
                
                // Always notify to update streams list
                let _ = this.update(cx, |_, cx| {
                    if changed {
                        cx.emit(AudioStateChanged { state: new_state });
                    }
                    cx.notify();
                });
            }
            
            eprintln!("[AudioService] PipeWire connection lost, reconnecting...");
            cx.background_executor().timer(std::time::Duration::from_secs(2)).await;
        }
    }

    pub fn state(&self) -> AudioState {
        self.state.read().clone()
    }

    pub fn set_sink_volume(&self, volume: u8, cx: &mut Context<Self>) {
        let volume = volume.min(100);
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{volume}%")])
                .output()
                .await;
        }).detach();
    }

    pub fn toggle_sink_mute(&self, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .output()
                .await;
        }).detach();
    }

    pub fn set_source_volume(&self, volume: u8, cx: &mut Context<Self>) {
        let volume = volume.min(100);
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SOURCE@", &format!("{volume}%")])
                .output()
                .await;
        }).detach();
    }

    pub fn set_default_sink(&self, id: u32, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-default", &id.to_string()])
                .output()
                .await;
        }).detach();
    }

    pub fn set_default_source(&self, id: u32, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-default", &id.to_string()])
                .output()
                .await;
        }).detach();
    }

    pub fn sinks(&self) -> Vec<AudioDevice> {
        self.sinks.read().clone()
    }

    pub fn sources(&self) -> Vec<AudioDevice> {
        self.sources.read().clone()
    }

    pub fn sink_inputs(&self) -> Vec<AudioStream> {
        self.sink_inputs.read().clone()
    }

    pub fn source_outputs(&self) -> Vec<AudioStream> {
        self.source_outputs.read().clone()
    }
}

// Global accessor
struct GlobalAudioService(Entity<AudioService>);
impl Global for GlobalAudioService {}

impl AudioService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalAudioService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalAudioService(service.clone()));
        service
    }
}
