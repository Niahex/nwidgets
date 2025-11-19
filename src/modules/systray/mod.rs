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
                // Mapper l'item du systray à une icône appropriée
                let icon = Self::get_icon_for_item(item);

                div()
                    .w_8()
                    .h_8()
                    .rounded_sm()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .text_color(rgb(SNOW0))
                    .child(icon)
            }))
    }

    fn get_icon_for_item(item: &TrayItem) -> &'static str {
        // Chercher dans le titre ou l'id (en minuscules)
        let search_str = format!("{} {}", item.title.to_lowercase(), item.id.to_lowercase());

        match search_str.as_str() {
            s if s.contains("steam") => icons::STEAM,
            s if s.contains("vesktop") || s.contains("discord") => icons::VESKTOP,
            s if s.contains("firefox") => icons::FIREFOX,
            s if s.contains("vlc") => icons::VLC,
            s if s.contains("1password") || s.contains("keepass") || s.contains("bitwarden") => {
                icons::PASSWORD
            }
            s if s.contains("obs") || s.contains("stream") => icons::STREAM,
            s if s.contains("bluetooth") => icons::BLUETOOTH_ON,
            s if s.contains("volume") || s.contains("sound") || s.contains("audio") => {
                icons::VOLUME_HIGH
            }
            s if s.contains("network") || s.contains("wifi") => icons::WIFI,
            s if s.contains("battery") || s.contains("power") => icons::BATTERY_FULL,
            s if s.contains("notification") => icons::BELL,
            _ => {
                // Fallback: icône par défaut pour le systray
                icons::SYSTRAY
            }
        }
    }
}
