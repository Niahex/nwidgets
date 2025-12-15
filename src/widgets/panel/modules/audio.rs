use gpui::prelude::*;
use gpui::*;
use crate::services::audio::{AudioService, AudioStateChanged};

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
        let audio = self.audio.clone();

        let icon = if state.sink_muted {
            "🔇"
        } else if state.sink_volume > 66 {
            "🔊"
        } else if state.sink_volume > 33 {
            "🔉"
        } else {
            "🔈"
        };

        div()
            .id("audio-module")
            .flex()
            .gap_2()
            .items_center()
            .px_2()
            .py_1()
            .rounded_md()
            .hover(|style| style.bg(rgb(0x313244)))
            .cursor_pointer()
            .on_click(move |_event, _window, cx| {
                audio.read(cx).toggle_sink_mute();
            })
            .child(icon)
            .child(format!("{}%", state.sink_volume))
    }
}
