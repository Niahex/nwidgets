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

#[derive(Clone)]
pub struct AudioStateChanged {
    pub state: AudioState,
}

pub struct AudioService {
    state: Arc<RwLock<AudioState>>,
}

impl EventEmitter<AudioStateChanged> for AudioService {}

impl AudioService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let initial_state = Self::get_initial_state();
        let state = Arc::new(RwLock::new(initial_state));
        
        let state_clone = Arc::clone(&state);
        
        // Monitor PipeWire events
        cx.spawn(async move |this, cx| {
            Self::monitor_pipewire(this, state_clone, cx).await
        }).detach();

        Self { state }
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

    async fn monitor_pipewire(
        this: gpui::WeakEntity<Self>,
        state: Arc<RwLock<AudioState>>,
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
                
                let changed = {
                    let mut current = state.write();
                    let changed = *current != new_state;
                    if changed {
                        *current = new_state.clone();
                    }
                    changed
                };
                
                if changed {
                    let _ = this.update(cx, |_, cx| {
                        cx.emit(AudioStateChanged { state: new_state });
                        cx.notify();
                    });
                }
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
