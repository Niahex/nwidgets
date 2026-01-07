use crate::services::audio::{AudioService, AudioStateChanged};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;

pub struct SourceModule {
    audio: Entity<AudioService>,
}

impl SourceModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);

        cx.subscribe(&audio, |_this, _audio, _event: &AudioStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { audio }
    }
}

impl Render for SourceModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.audio.read(cx).state();

        let icon_name = if state.source_muted {
            "source-muted"
        } else if state.source_volume > 66 {
            "source-high"
        } else if state.source_volume > 33 {
            "source-medium"
        } else if state.source_volume > 0 {
            "source-low"
        } else {
            "source-zero"
        };

        Icon::new(icon_name)
            .size(px(16.))
            .color(cx.global::<crate::theme::Theme>().text)
    }
}
