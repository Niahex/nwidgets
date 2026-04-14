use crate::assets::Icon;
use crate::services::media::audio::{AudioService, AudioState, AudioStateChanged};
use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

pub enum VolumeType {
    Sink,
    Source,
}

pub struct AudioVolumeModule {
    audio: Entity<AudioService>,
    volume_type: VolumeType,
}

impl AudioVolumeModule {
    pub fn new(volume_type: VolumeType, cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);

        cx.subscribe(&audio, |_this, _audio, _event: &AudioStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { audio, volume_type }
    }

    pub fn sink(cx: &mut Context<Self>) -> Self {
        Self::new(VolumeType::Sink, cx)
    }

    pub fn source(cx: &mut Context<Self>) -> Self {
        Self::new(VolumeType::Source, cx)
    }

    fn get_icon_name(&self, state: &AudioState) -> &'static str {
        match self.volume_type {
            VolumeType::Sink => {
                if state.sink_muted {
                    "sink-muted"
                } else if state.sink_volume > 66 {
                    "sink-high"
                } else if state.sink_volume > 33 {
                    "sink-medium"
                } else if state.sink_volume > 0 {
                    "sink-low"
                } else {
                    "sink-zero"
                }
            }
            VolumeType::Source => {
                if state.source_muted {
                    "source-muted"
                } else if state.source_volume > 66 {
                    "source-high"
                } else if state.source_volume > 33 {
                    "source-medium"
                } else if state.source_volume > 0 {
                    "source-low"
                } else {
                    "source-zero"
                }
            }
        }
    }
}

impl Render for AudioVolumeModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.audio.read(cx).state();
        let icon_name = self.get_icon_name(&state);

        Icon::new(icon_name).size(px(16.)).color(cx.theme().text)
    }
}
