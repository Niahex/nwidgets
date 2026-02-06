pub const SINK_MUTED: &str = "sink-muted";
pub const SINK_ZERO: &str = "sink-zero";
pub const SINK_LOW: &str = "sink-low";
pub const SINK_MEDIUM: &str = "sink-medium";
pub const SINK_HIGH: &str = "sink-high";

pub const OSD_DISPLAY_DURATION_MS: u64 = 2000;
pub const ANIMATION_FRAME_MS: u64 = 8;
pub const ANIMATION_SMOOTHNESS: f32 = 0.7;

#[derive(Debug, Clone, PartialEq)]
pub enum OsdEvent {
    Volume(String, u8, bool),
    Microphone(bool),
    CapsLock(bool),
    Clipboard,
}

#[derive(Clone)]
pub struct OsdStateChanged {
    pub event: Option<OsdEvent>,
    pub visible: bool,
}
