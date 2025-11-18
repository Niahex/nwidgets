use crate::services::systray::{SystemTrayService, TrayItem};
use crate::theme::*;
use gpui::{div, prelude::*, rgb};

pub struct SystrayModule {
    items: Vec<TrayItem>,
}

impl SystrayModule {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn update(&mut self, items: Vec<TrayItem>) {
        self.items = items;
    }

    /// Start monitoring system tray - returns a future that resolves to tray items
    pub async fn start_monitoring() -> Result<Vec<TrayItem>, Box<dyn std::error::Error>> {
        let mut service = SystemTrayService::new();
        service.start_monitoring().await.map_err(|e| e.into())
    }

    pub fn render(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .children(self.items.iter().map(|item| {
                div()
                    .w_8()
                    .h_8()
                    .bg(rgb(POLAR2))
                    .rounded_sm()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .child(if !item.icon_name.is_empty() {
                        item.title.chars().next().unwrap_or('?').to_string()
                    } else {
                        "•".to_string()
                    })
            }))
    }
}
