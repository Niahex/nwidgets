use std::sync::Arc;
use parking_lot::RwLock;
use wayland_client::{Connection, QueueHandle, Dispatch, Proxy};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerShellLayer {
    Background,
    Bottom,
    Top,
    Overlay,
}

impl LayerShellLayer {
    pub fn to_wlr(&self) -> zwlr_layer_shell_v1::Layer {
        match self {
            LayerShellLayer::Background => zwlr_layer_shell_v1::Layer::Background,
            LayerShellLayer::Bottom => zwlr_layer_shell_v1::Layer::Bottom,
            LayerShellLayer::Top => zwlr_layer_shell_v1::Layer::Top,
            LayerShellLayer::Overlay => zwlr_layer_shell_v1::Layer::Overlay,
        }
    }
}

pub struct LayerShellAnchor;
impl LayerShellAnchor {
    pub const NONE: u32 = 0;
    pub const TOP: u32 = 1;
    pub const BOTTOM: u32 = 2;
    pub const LEFT: u32 = 4;
    pub const RIGHT: u32 = 8;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerShellKeyboardInteractivity {
    None,
    Exclusive,
    OnDemand,
}

impl LayerShellKeyboardInteractivity {
    pub fn to_wlr(&self) -> zwlr_layer_surface_v1::KeyboardInteractivity {
        match self {
            LayerShellKeyboardInteractivity::None => zwlr_layer_surface_v1::KeyboardInteractivity::None,
            LayerShellKeyboardInteractivity::Exclusive => zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
            LayerShellKeyboardInteractivity::OnDemand => zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayerShellConfig {
    pub layer: LayerShellLayer,
    pub anchor: u32,
    pub exclusive_zone: Option<i32>,
    pub namespace: String,
    pub keyboard_interactivity: LayerShellKeyboardInteractivity,
    pub margin: (i32, i32, i32, i32),
}

#[derive(Clone)]
pub struct LayerShellService {
}

impl LayerShellService {
    pub fn new() -> Self {
        log::info!("Initializing LayerShellService");
        Self {}
    }
    
    pub fn set_input_region(&self, enable: bool) {
        log::info!("LayerShellService::set_input_region({})", enable);
    }
    
    pub fn configure_layer_shell(&self, config: &LayerShellConfig) {
        log::info!("LayerShellService::configure_layer_shell({:?})", config.namespace);
    }
}
