use crate::theme::*;
use chrono::Local;
use gpui::{div, prelude::*, rgb};

pub struct DateTimeModule;

impl DateTimeModule {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self) -> impl IntoElement {
        let now = Local::now();

        div()
            .w_16()
            .h_16()
            .rounded_md()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_0p5()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(SNOW2))
                    .child(now.format("%H:%M").to_string())
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(SNOW0))
                    .child(now.format("%d/%m/%y").to_string())
            )
    }
}
