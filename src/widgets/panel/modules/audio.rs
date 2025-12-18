use gpui::prelude::*;
use gpui::*;
use crate::services::audio::{AudioService, AudioStateChanged};
use crate::utils::Icon;

pub struct AudioModule {
    audio: Entity<AudioService>,
}

impl AudioModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);

        // Subscribe to audio state changes
        cx.subscribe(&audio, |_this, _audio, _event: &AudioStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { audio }
    }
}

impl Render for AudioModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.audio.read(cx).state();

        let icon_name = if state.sink_muted {
            "sink-muted"
        } else if state.sink_volume > 66 {
            "sink-high"
        } else if state.sink_volume > 33 {
            "sink-medium"
        } else if state.sink_volume > 0 {
            "sink-low"
        } else {
            "sink-zero"
        };

        Icon::new(icon_name)
            .size(px(16.))
            .color(rgb(0xeceff4))
    }
}
