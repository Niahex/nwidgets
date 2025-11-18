use crate::theme::*;
use gpui::{div, prelude::*, rgb};

pub struct VolumeModule {
    volume: u8,
}

impl VolumeModule {
    pub fn new(volume: u8) -> Self {
        Self { volume }
    }

    pub fn update(&mut self, volume: u8) {
        self.volume = volume;
    }

    pub fn render(&self) -> impl IntoElement {
        let volume_icon = if self.volume == 0 {
            "🔇"
        } else if self.volume < 50 {
            "🔉"
        } else {
            "🔊"
        };

        div()
            .w_16()
            .h_8()
            .bg(rgb(GREEN))
            .rounded_md()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap_1()
            .text_color(rgb(POLAR0))
            .text_xs()
            .child(volume_icon)
            .child(format!("{}%", self.volume))
    }
}
