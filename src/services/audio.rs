use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, SharedString, WeakEntity};
use parking_lot::RwLock;
use pipewire as pw;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Debug, Clone, PartialEq)]
pub struct AudioDevice {
    pub id: u32,
    pub name: SharedString,
    pub description: SharedString,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioStream {
    pub id: u32,
    pub app_name: SharedString,
    pub window_title: Option<SharedString>,
    pub volume: u8,
    pub muted: bool,
}

#[derive(Clone)]
pub struct AudioStateChanged {
    pub state: AudioState,
}

#[derive(Debug, Clone)]
struct PwNodeInfo {
    id: u32,
    name: SharedString,
    description: SharedString,
    media_class: SharedString,
}

pub struct AudioService {
    state: Arc<RwLock<AudioState>>,
    sinks: Arc<RwLock<Vec<AudioDevice>>>,
    sources: Arc<RwLock<Vec<AudioDevice>>>,
    sink_inputs: Arc<RwLock<Vec<AudioStream>>>,
    source_outputs: Arc<RwLock<Vec<AudioStream>>>,
}

impl EventEmitter<AudioStateChanged> for AudioService {}

enum AudioUpdate {
    State(AudioState),
    Devices {
        sinks: Vec<AudioDevice>,
        sources: Vec<AudioDevice>,
        sink_inputs: Vec<AudioStream>,
        source_outputs: Vec<AudioStream>,
    },
}

impl AudioService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = Arc::new(RwLock::new(AudioState::default()));
        let sinks = Arc::new(RwLock::new(Vec::new()));
        let sources = Arc::new(RwLock::new(Vec::new()));
        let sink_inputs = Arc::new(RwLock::new(Vec::new()));
        let source_outputs = Arc::new(RwLock::new(Vec::new()));

        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<AudioUpdate>();

        // 1. Worker Task (Tokio)
        gpui_tokio::Tokio::spawn(cx, async move { Self::audio_worker(ui_tx).await }).detach();

        // 2. UI Task (GPUI)
        let state_clone = Arc::clone(&state);
        let sinks_clone = Arc::clone(&sinks);
        let sources_clone = Arc::clone(&sources);
        let sink_inputs_clone = Arc::clone(&sink_inputs);
        let source_outputs_clone = Arc::clone(&source_outputs);

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(update) = ui_rx.next().await {
                    match update {
                        AudioUpdate::State(new_state) => {
                            let mut changed = false;
                            {
                                let mut current = state_clone.write();
                                if *current != new_state {
                                    *current = new_state.clone();
                                    changed = true;
                                }
                            }
                            if changed {
                                let _ = this.update(&mut cx, |_, cx| {
                                    cx.emit(AudioStateChanged { state: new_state });
                                    cx.notify();
                                });
                            }
                        }
                        AudioUpdate::Devices {
                            sinks,
                            sources,
                            sink_inputs,
                            source_outputs,
                        } => {
                            *sinks_clone.write() = sinks;
                            *sources_clone.write() = sources;
                            *sink_inputs_clone.write() = sink_inputs;
                            *source_outputs_clone.write() = source_outputs;

                            let _ = this.update(&mut cx, |_, cx| {
                                cx.notify();
                            });
                        }
                    }
                }
            }
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

    // Helper async function to get volume via wpctl process
    async fn get_volume_wpctl_async(device: &str) -> (u8, bool) {
        if let Ok(output) = tokio::process::Command::new("wpctl")
            .args(["get-volume", device])
            .output()
            .await
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                let muted = text.contains("[MUTED]");
                if let Some(vol_str) = text.split_whitespace().nth(1) {
                    if let Ok(vol) = vol_str.parse::<f32>() {
                        return ((vol * 100.0).round() as u8, muted);
                    }
                }
            }
        }
        (50, false)
    }

    // Get default sink/source ID via wpctl inspect asynchronously
    async fn get_default_device_id_async(device: &str) -> Option<u32> {
        if let Ok(output) = tokio::process::Command::new("wpctl")
            .args(["inspect", device])
            .output()
            .await
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                // First line: "id 76, type PipeWire:Interface:Node"
                if let Some(first_line) = text.lines().next() {
                    if let Some(id_part) = first_line.strip_prefix("id ") {
                        if let Some(id_str) = id_part.split(',').next() {
                            return id_str.trim().parse().ok();
                        }
                    }
                }
            }
        }
        None
    }

    // Worker running in Tokio context, Send-safe (no GPUI types)
    async fn audio_worker(ui_tx: futures::channel::mpsc::UnboundedSender<AudioUpdate>) {
        loop {
            let (tx, mut rx) = futures::channel::mpsc::unbounded::<PwEvent>();
            let nodes_data: Arc<RwLock<HashMap<u32, PwNodeInfo>>> =
                Arc::new(RwLock::new(HashMap::new()));
            let nodes_data_thread = Arc::clone(&nodes_data);

            std::thread::spawn({
                let tx = tx;
                move || {
                    pw::init();

                    let mainloop = match pw::main_loop::MainLoopRc::new(None) {
                        Ok(ml) => ml,
                        Err(e) => {
                            log::error!("Failed to create PipeWire mainloop: {e}");
                            return;
                        }
                    };

                    let context = match pw::context::ContextRc::new(&mainloop, None) {
                        Ok(ctx) => ctx,
                        Err(e) => {
                            log::error!("Failed to create PipeWire context: {e}");
                            return;
                        }
                    };

                    let core = match context.connect_rc(None) {
                        Ok(c) => c,
                        Err(e) => {
                            log::error!("Failed to connect to PipeWire: {e}");
                            return;
                        }
                    };

                    let registry = match core.get_registry_rc() {
                        Ok(r) => r,
                        Err(e) => {
                            log::error!("Failed to get PipeWire registry: {e}");
                            return;
                        }
                    };

                    let nodes: std::rc::Rc<std::cell::RefCell<Vec<pw::node::Node>>> =
                        std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
                    let listeners: std::rc::Rc<std::cell::RefCell<Vec<pw::node::NodeListener>>> =
                        std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));

                    let nodes_clone = nodes.clone();
                    let listeners_clone = listeners.clone();
                    let registry_clone = registry.clone();
                    let nodes_data_clone = nodes_data_thread.clone();
                    let tx_remove = tx.clone();

                    let _registry_listener = registry
                        .add_listener_local()
                        .global(move |global| {
                            if global.type_ == pw::types::ObjectType::Node {
                                if let Some(props) = &global.props {
                                    let media_class = props.get("media.class").unwrap_or("");
                                    let node_name = props.get("node.name").unwrap_or("");
                                    let node_desc = props
                                        .get("node.description")
                                        .or_else(|| props.get("node.nick"))
                                        .unwrap_or(node_name);

                                    let dominated = media_class.contains("Audio/Sink")
                                        || media_class.contains("Audio/Source")
                                        || media_class.contains("Stream/");

                                    if dominated {
                                        let info = PwNodeInfo {
                                            id: global.id,
                                            name: node_name.to_string().into(),
                                            description: node_desc.to_string().into(),
                                            media_class: media_class.to_string().into(),
                                        };
                                        nodes_data_clone.write().insert(global.id, info);
                                        let _ = tx.unbounded_send(PwEvent::NodeAdded);

                                        if let Ok(node) =
                                            registry_clone.bind::<pw::node::Node, _>(global)
                                        {
                                            let tx_param = tx.clone();

                                            let node_listener = node
                                                .add_listener_local()
                                                .param(move |_, _, _, _, _| {
                                                    let _ = tx_param
                                                        .unbounded_send(PwEvent::ParamChanged);
                                                })
                                                .register();

                                            node.subscribe_params(&[
                                                pw::spa::param::ParamType::Props,
                                            ]);

                                            nodes_clone.borrow_mut().push(node);
                                            listeners_clone.borrow_mut().push(node_listener);
                                        }
                                    }
                                }
                            }
                        })
                        .global_remove(move |id| {
                            let _ = tx_remove.unbounded_send(PwEvent::NodeRemoved(id));
                        })
                        .register();

                    mainloop.run();
                }
            });

            // Initial fetch
            let (sink_vol, sink_muted) = Self::get_volume_wpctl_async("@DEFAULT_AUDIO_SINK@").await;
            let (source_vol, source_muted) =
                Self::get_volume_wpctl_async("@DEFAULT_AUDIO_SOURCE@").await;

            let _ = ui_tx.unbounded_send(AudioUpdate::State(AudioState {
                sink_volume: sink_vol,
                sink_muted,
                source_volume: source_vol,
                source_muted,
            }));

            let mut last_update = std::time::Instant::now();
            let debounce = std::time::Duration::from_millis(50);

            while let Some(event) = rx.next().await {
                if let PwEvent::NodeRemoved(id) = event {
                    nodes_data.write().remove(&id);
                }

                let now = std::time::Instant::now();
                if now.duration_since(last_update) < debounce {
                    while rx.try_next().is_ok() {}
                    tokio::time::sleep(debounce).await;
                }
                last_update = std::time::Instant::now();

                // Update volumes
                let (sink_vol, sink_muted) =
                    Self::get_volume_wpctl_async("@DEFAULT_AUDIO_SINK@").await;
                let (source_vol, source_muted) =
                    Self::get_volume_wpctl_async("@DEFAULT_AUDIO_SOURCE@").await;

                let _ = ui_tx.unbounded_send(AudioUpdate::State(AudioState {
                    sink_volume: sink_vol,
                    sink_muted,
                    source_volume: source_vol,
                    source_muted,
                }));

                // Get default device IDs
                let default_sink_id =
                    Self::get_default_device_id_async("@DEFAULT_AUDIO_SINK@").await;
                let default_source_id =
                    Self::get_default_device_id_async("@DEFAULT_AUDIO_SOURCE@").await;

                // Build device lists
                let nodes_snapshot = nodes_data.read();
                let mut new_sinks = Vec::new();
                let mut new_sources = Vec::new();
                let mut new_sink_inputs = Vec::new();
                let mut new_source_outputs = Vec::new();

                for info in nodes_snapshot.values() {
                    if info.media_class.contains("Audio/Sink")
                        && !info.media_class.contains("Stream")
                    {
                        new_sinks.push(AudioDevice {
                            id: info.id,
                            name: info.name.clone(),
                            description: info.description.clone(),
                            is_default: default_sink_id == Some(info.id),
                        });
                    } else if info.media_class.contains("Audio/Source")
                        && !info.media_class.contains("Stream")
                    {
                        new_sources.push(AudioDevice {
                            id: info.id,
                            name: info.name.clone(),
                            description: info.description.clone(),
                            is_default: default_source_id == Some(info.id),
                        });
                    } else if info.media_class.contains("Stream/Output/Audio") {
                        new_sink_inputs.push(AudioStream {
                            id: info.id,
                            app_name: info.description.clone(),
                            window_title: None,
                            volume: 100,
                            muted: false,
                        });
                    } else if info.media_class.contains("Stream/Input/Audio") {
                        new_source_outputs.push(AudioStream {
                            id: info.id,
                            app_name: info.description.clone(),
                            window_title: None,
                            volume: 100,
                            muted: false,
                        });
                    }
                }
                drop(nodes_snapshot);

                let _ = ui_tx.unbounded_send(AudioUpdate::Devices {
                    sinks: new_sinks,
                    sources: new_sources,
                    sink_inputs: new_sink_inputs,
                    source_outputs: new_source_outputs,
                });
            }

            log::warn!("PipeWire connection lost, reconnecting...");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
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
        })
        .detach();
    }

    #[allow(dead_code)]
    pub fn toggle_sink_mute(&self, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .output()
                .await;
        })
        .detach();
    }

    pub fn set_source_volume(&self, volume: u8, cx: &mut Context<Self>) {
        let volume = volume.min(100);
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args([
                    "set-volume",
                    "@DEFAULT_AUDIO_SOURCE@",
                    &format!("{volume}%"),
                ])
                .output()
                .await;
        })
        .detach();
    }

    pub fn set_default_sink(&self, id: u32, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-default", &id.to_string()])
                .output()
                .await;
        })
        .detach();
    }

    pub fn set_default_source(&self, id: u32, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = tokio::process::Command::new("wpctl")
                .args(["set-default", &id.to_string()])
                .output()
                .await;
        })
        .detach();
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

#[derive(Debug)]
enum PwEvent {
    NodeAdded,
    NodeRemoved(u32),
    ParamChanged,
}

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
