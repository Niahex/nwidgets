use std::sync::Arc;
use parking_lot::RwLock;

use crate::TOKIO_RUNTIME;

#[derive(Clone, Debug, Default)]
pub struct AudioSink {
    pub name: String,
    pub description: String,
    pub volume: f32,
    pub is_muted: bool,
    pub is_default: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AudioSource {
    pub name: String,
    pub description: String,
    pub volume: f32,
    pub is_muted: bool,
    pub is_default: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AudioStream {
    pub name: String,
    pub app_name: String,
    pub volume: f32,
    pub is_muted: bool,
}

#[derive(Clone)]
pub struct AudioService {
    state: Arc<RwLock<AudioState>>,
}

#[derive(Default)]
struct AudioState {
    sinks: Vec<AudioSink>,
    sources: Vec<AudioSource>,
    streams: Vec<AudioStream>,
    default_sink_volume: f32,
    default_sink_muted: bool,
    default_source_volume: f32,
    default_source_muted: bool,
}

impl AudioService {
    pub fn new() -> Self {
        let service = Self {
            state: Arc::new(RwLock::new(AudioState::default())),
        };

        service.start();
        service
    }

    fn start(&self) {
        let _state = self.state.clone();

        TOKIO_RUNTIME.spawn(async move {
            log::info!("Audio service started (stub)");
        });
    }

    pub fn get_sink_volume(&self) -> f32 {
        self.state.read().default_sink_volume
    }

    pub fn is_sink_muted(&self) -> bool {
        self.state.read().default_sink_muted
    }

    pub fn get_source_volume(&self) -> f32 {
        self.state.read().default_source_volume
    }

    pub fn is_source_muted(&self) -> bool {
        self.state.read().default_source_muted
    }

    pub fn set_sink_volume(&self, volume: f32) {
        self.state.write().default_sink_volume = volume;
    }

    pub fn set_sink_muted(&self, muted: bool) {
        self.state.write().default_sink_muted = muted;
    }

    pub fn set_source_volume(&self, volume: f32) {
        self.state.write().default_source_volume = volume;
    }

    pub fn set_source_muted(&self, muted: bool) {
        self.state.write().default_source_muted = muted;
    }
}
