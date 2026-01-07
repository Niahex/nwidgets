use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Arc;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioStream {
    pub id: u32,
    pub app_name: String,
    pub volume: u8,
    pub muted: bool,
    pub window_title: Option<String>,
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
        let (initial_state, initial_sinks, initial_sources, initial_sink_inputs, initial_source_outputs) = Self::fetch_full_state();
        
        let state = Arc::new(RwLock::new(initial_state));
        let sinks = Arc::new(RwLock::new(initial_sinks));
        let sources = Arc::new(RwLock::new(initial_sources));
        let sink_inputs = Arc::new(RwLock::new(initial_sink_inputs));
        let source_outputs = Arc::new(RwLock::new(initial_source_outputs));

        let state_clone = Arc::clone(&state);
        let sinks_clone = Arc::clone(&sinks);
        let sources_clone = Arc::clone(&sources);
        let sink_inputs_clone = Arc::clone(&sink_inputs);
        let source_outputs_clone = Arc::clone(&source_outputs);

        // Spawn background task to monitor PipeWire events with pw-mon
        cx.spawn(async move |this, cx| {
            Self::monitor_audio_events(this, state_clone, sinks_clone, sources_clone, sink_inputs_clone, source_outputs_clone, cx).await
        })
        .detach();

        Self {
            state,
            sinks,
            sources,
            sink_inputs,
            source_outputs,
        }
    }

    pub fn state(&self) -> AudioState {
        self.state.read().clone()
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

    pub fn set_sink_volume(&self, volume: u8, _cx: &mut Context<Self>) {
        let _volume = volume.min(100);
        // Commande wpctl désactivée pour test
        // gpui_tokio::Tokio::spawn(cx, async move {
        //     let _ = tokio::process::Command::new("wpctl")
        //         .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{volume}%")])
        //         .output()
        //         .await;
        // }).detach();
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

    pub fn toggle_sink_mute(&self, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .output()
                .await;
        }).detach();
    }

    pub fn toggle_source_mute(&self, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
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

    async fn monitor_audio_events(
        this: WeakEntity<Self>,
        state: Arc<RwLock<AudioState>>,
        sinks: Arc<RwLock<Vec<AudioDevice>>>,
        sources: Arc<RwLock<Vec<AudioDevice>>>,
        sink_inputs: Arc<RwLock<Vec<AudioStream>>>,
        source_outputs: Arc<RwLock<Vec<AudioStream>>>,
        cx: &mut AsyncApp,
    ) {
        loop {
            let (tx, mut rx) = futures::channel::mpsc::unbounded();

            // Spawn pw-mon in background thread
            let tx_clone = tx.clone();
            std::thread::spawn(move || {
                let mut child = match Command::new("pw-mon").stdout(Stdio::piped()).spawn() {
                    Ok(child) => child,
                    Err(e) => {
                        eprintln!("Failed to start pw-mon: {e}. Falling back to polling.");
                        loop {
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            if tx_clone.unbounded_send(()).is_err() {
                                break;
                            }
                        }
                        return;
                    }
                };

                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        if line.trim().starts_with("changed:")
                            && tx_clone.unbounded_send(()).is_err() {
                                break;
                            }
                    }
                }

                let _ = child.kill();
            });

            // Process events with debouncing
            let mut last_update = std::time::Instant::now();
            let debounce_duration = std::time::Duration::from_millis(50); // Increased slightly for better efficiency

            while let Some(()) = rx.next().await {
                let now = std::time::Instant::now();
                if now.duration_since(last_update) < debounce_duration {
                    while let Ok(Some(())) = rx.try_next() {}
                    cx.background_executor()
                        .timer(debounce_duration - now.duration_since(last_update))
                        .await;
                }

                last_update = std::time::Instant::now();

                // Optimized: Fetch everything in one go
                let (new_state, new_sinks, new_sources, new_sink_inputs, new_source_outputs) = Self::fetch_full_state();

                let state_changed = {
                    let mut current_state = state.write();
                    let changed = *current_state != new_state;
                    if changed {
                        *current_state = new_state.clone();
                    }
                    changed
                };

                let devices_changed = {
                    let mut current_sinks = sinks.write();
                    let mut current_sources = sources.write();
                    let changed = *current_sinks != new_sinks || *current_sources != new_sources;
                    if changed {
                        *current_sinks = new_sinks;
                        *current_sources = new_sources;
                    }
                    changed
                };

                let streams_changed = {
                    let mut current_sink_inputs = sink_inputs.write();
                    let mut current_source_outputs = source_outputs.write();
                    let changed = *current_sink_inputs != new_sink_inputs || *current_source_outputs != new_source_outputs;
                    if changed {
                        *current_sink_inputs = new_sink_inputs;
                        *current_source_outputs = new_source_outputs;
                    }
                    changed
                };

                if state_changed || devices_changed || streams_changed {
                    let _ = this.update(cx, |_, cx| {
                        if state_changed {
                            cx.emit(AudioStateChanged { state: new_state });
                        }
                        cx.notify();
                    });
                }
            }

            eprintln!("pw-mon process ended, restarting in 2 seconds...");
            cx.background_executor()
                .timer(std::time::Duration::from_secs(2))
                .await;
        }
    }

    fn fetch_full_state() -> (AudioState, Vec<AudioDevice>, Vec<AudioDevice>, Vec<AudioStream>, Vec<AudioStream>) {
        // 1. Fetch volume/mute via wpctl (faster than parsing large pw-dump for this)
        let state = Self::fetch_audio_state();

        // 2. Fetch everything else via a SINGLE pw-dump call
        let output = Command::new("pw-dump")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let nodes: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap_or_default();
        
        let mut sinks = Vec::new();
        let mut sources = Vec::new();
        let mut sink_inputs = Vec::new();
        let mut source_outputs = Vec::new();

        for node in &nodes {
            if node["type"] != "PipeWire:Interface:Node" {
                continue;
            }

            let props = &node["info"]["props"];
            let media_class = props["media.class"].as_str().unwrap_or_default();
            let id = node["id"].as_u64().unwrap_or(0) as u32;

            match media_class {
                "Audio/Sink" => {
                    let desc = props["node.description"]
                        .as_str()
                        .or_else(|| props["node.name"].as_str())
                        .unwrap_or("Unknown Sink")
                        .to_string();
                    sinks.push(AudioDevice { id, name: desc.clone(), description: desc, is_default: false });
                }
                "Audio/Source" => {
                    let desc = props["node.description"]
                        .as_str()
                        .or_else(|| props["node.name"].as_str())
                        .unwrap_or("Unknown Source")
                        .to_string();
                    sources.push(AudioDevice { id, name: desc.clone(), description: desc, is_default: false });
                }
                "Stream/Output/Audio" => {
                    if let Some(stream) = Self::parse_stream(node, true) {
                        sink_inputs.push(stream);
                    }
                }
                "Stream/Input/Audio" => {
                    if let Some(stream) = Self::parse_stream(node, false) {
                        source_outputs.push(stream);
                    }
                }
                _ => {}
            }
        }

        (state, sinks, sources, sink_inputs, source_outputs)
    }

    fn parse_stream(node: &serde_json::Value, is_input: bool) -> Option<AudioStream> {
        let id = node["id"].as_u64()? as u32;
        let props = &node["info"]["props"];
        
        let app_name = props["application.name"]
            .as_str()
            .or_else(|| props["node.name"].as_str())?
            .to_string();
        
        let params = &node["info"]["params"]["Props"];
        let vol_muted = params.as_array().and_then(|arr| arr.first());
        
        let volume = vol_muted
            .and_then(|p| p["channelVolumes"].as_array())
            .and_then(|vols| vols.first())
            .and_then(|v| v.as_f64())
            .map(|v| (v * 100.0) as u8)
            .unwrap_or(100);
        
        let muted = vol_muted
            .and_then(|p| p["mute"].as_bool())
            .unwrap_or(false);
        
        let window_title = props["media.name"]
            .as_str()
            .or_else(|| props["node.description"].as_str())
            .map(|s| s.to_string());
        
        Some(AudioStream {
            id,
            app_name,
            volume,
            muted,
            window_title,
        })
    }

    fn fetch_audio_state() -> AudioState {
        let sink_output = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let source_output = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let parse_volume = |s: &str| -> (u8, bool) {
            let muted = s.contains("[MUTED]");
            let volume = s
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse::<f32>().ok())
                .map(|v| (v * 100.0) as u8)
                .unwrap_or(50);
            (volume, muted)
        };

        let (sink_volume, sink_muted) = parse_volume(&sink_output);
        let (source_volume, source_muted) = parse_volume(&source_output);

        AudioState {
            sink_volume,
            sink_muted,
            source_volume,
            source_muted,
        }
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