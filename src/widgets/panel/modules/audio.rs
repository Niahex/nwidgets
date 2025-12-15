use gpui::prelude::*;
use gpui::*;
use crate::services::audio::{AudioService, AudioStateChanged};
use crate::utils::{Icon, IconName};

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

        let icon_name = if state.sink_muted {
            IconName::SinkMuted
        } else if state.sink_volume > 66 {
            IconName::SinkHigh
        } else if state.sink_volume > 33 {
            IconName::SinkMedium
        } else if state.sink_volume > 0 {
            IconName::SinkLow
        } else {
            IconName::SinkZero
        };

        div()
            .id("audio-module")
            .flex()
            .gap_2()
            .items_center()
            .px_3()
            .py_2()
            .rounded_md()
            .text_sm()
            .hover(|style| style.bg(rgba(0x4c566a80))) // $polar3 with opacity
            .cursor_pointer()
            .on_click(move |_event, _window, cx| {
                audio.read(cx).toggle_sink_mute();
            })
            .child(
                Icon::new(icon_name)
                    .size(px(18.))
                    .color(rgb(0xeceff4)) // $snow2
            )
            .child(
                div()
                    .text_color(rgb(0xeceff4)) // $snow2
                    .child(format!("{}%", state.sink_volume))
            )
    }
}
