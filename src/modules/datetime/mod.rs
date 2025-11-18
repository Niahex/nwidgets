use crate::theme::*;
use gpui::{div, prelude::*, rgb};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DateTimeModule;

impl DateTimeModule {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self) -> impl IntoElement {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Add timezone offset for CET (UTC+1)
        let local_time = now + 3600;
        let hours = (local_time / 3600) % 24;
        let minutes = (local_time / 60) % 60;

        div()
            .w_16()
            .h_8()
            .rounded_md()
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(SNOW0))
            .text_sm()
            .child(format!("{:02}:{:02}", hours, minutes))
    }
}
