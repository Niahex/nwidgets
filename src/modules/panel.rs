use gpui::{Context, Window, div, prelude::*, rgb, rgba, Timer};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::process::Command;

pub struct Panel {
    enabled: bool,
    volume: u8,
}

impl Panel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let initial_volume = Self::get_volume_level();
        println!("[PANEL] Creating panel with initial volume: {}", initial_volume);
        
        let panel = Self {
            enabled: true,
            volume: initial_volume,
        };

        // Monitor volume changes every 100ms
        cx.spawn(async move |this, cx| {
            println!("[PANEL] Volume monitoring task started");
            loop {
                Timer::after(Duration::from_millis(100)).await;
                let new_volume = Self::get_volume_level();
                println!("[PANEL] Volume check: new={}", new_volume);
                let _ = this.update(cx, |panel, cx| {
                    println!("[PANEL] Panel volume state: {}", panel.volume);
                    if panel.volume != new_volume {
                        println!("[PANEL] Volume changed: {} -> {}", panel.volume, new_volume);
                        panel.volume = new_volume;
                        cx.notify();
                    }
                });
            }
        }).detach();

        panel
    }

    fn get_volume_level() -> u8 {
        let output = Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output();
            
        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                println!("[PANEL] wpctl output: '{}'", output_str.trim());
                
                if let Some(volume_str) = output_str.strip_prefix("Volume: ") {
                    if let Ok(volume_float) = volume_str.trim().parse::<f32>() {
                        let volume = (volume_float * 100.0) as u8;
                        println!("[PANEL] Parsed volume: {}% (from {})", volume, volume_float);
                        return volume;
                    }
                }
                println!("[PANEL] Failed to parse volume from: '{}'", output_str);
                50
            },
            Err(e) => {
                println!("[PANEL] wpctl command failed: {}", e);
                50
            }
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        println!("[PANEL] Rendering with volume: {}", self.volume);
        
        if !self.enabled {
            return div().size_full();
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hours = (now / 3600) % 24;
        let minutes = (now / 60) % 60;

        div()
            .size_full()
            .bg(rgba(0x1a1a1aaa))
            .border_b_1()
            .border_color(rgba(0x444444aa))
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .child(
                // Left side - App launcher / menu
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x333333aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("Menu")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x333333aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("Apps")
                    )
            )
            .child(
                // Center - Window title or workspace info
                div()
                    .text_color(rgb(0xffffff))
                    .text_sm()
                    .child("Workspace 1")
            )
            .child(
                // Right side - System tray, clock, etc.
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x333333aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child(if self.volume == 0 { "🔇" } else if self.volume < 50 { "🔉" } else { "🔊" })
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x333333aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("🔋")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgba(0x444444aa))
                            .rounded_md()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child(format!("{:02}:{:02}", hours, minutes))
                    )
            )
    }
}
