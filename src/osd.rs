use gpui::*;
use std::fs;
use std::time::{Duration, Instant};
use std::process::Command;

#[derive(Clone)]
pub enum OsdType {
    CapsLock(bool),
    NumLock(bool),
    Volume(u8),
    Microphone(bool),
}

pub struct Osd {
    osd_type: OsdType,
    visible: bool,
    last_caps_state: bool,
    last_volume: u8,
    show_until: Option<Instant>,
    volume_text: String,
}

impl Osd {
    pub fn new(osd_type: OsdType) -> Self {
        Self {
            osd_type,
            visible: false,
            last_caps_state: false,
            last_volume: 50,
            show_until: None,
            volume_text: String::new(),
        }
    }

    fn get_caps_lock_state() -> bool {
        if let Ok(content) = fs::read_to_string("/sys/class/leds/input0::capslock/brightness") {
            if let Ok(brightness) = content.trim().parse::<u8>() {
                return brightness > 0;
            }
        }
        false
    }

    fn get_volume_level() -> u8 {
        Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
            .and_then(|output| {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Parse "Volume: 0.35" format
                if let Some(volume_str) = output_str.strip_prefix("Volume: ") {
                    if let Ok(volume_float) = volume_str.trim().parse::<f32>() {
                        return Ok((volume_float * 100.0) as u8);
                    }
                }
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Parse error"))
            })
            .unwrap_or(50)
    }
}

impl Render for Osd {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let caps_on = Self::get_caps_lock_state();
        let volume = Self::get_volume_level();
        let now = Instant::now();
        
        // Show OSD when caps lock state changes
        if caps_on != self.last_caps_state {
            self.last_caps_state = caps_on;
            self.visible = true;
            self.show_until = Some(now + Duration::from_millis(2500));
            self.osd_type = OsdType::CapsLock(caps_on);
        }

        // Show OSD when volume changes
        if volume != self.last_volume {
            self.last_volume = volume;
            self.volume_text = format!("{}%", volume);
            self.visible = true;
            self.show_until = Some(now + Duration::from_millis(2500));
            self.osd_type = OsdType::Volume(volume);
        }

        // Hide after 2.5s
        if let Some(hide_time) = self.show_until {
            if now >= hide_time {
                self.visible = false;
                self.show_until = None;
            }
        }

        // Keep checking
        let entity = cx.entity_id();
        cx.defer(move |cx| {
            cx.notify(entity);
        });

        if !self.visible {
            return div();
        }

        let (icon, text, color) = match &self.osd_type {
            OsdType::CapsLock(enabled) => {
                let text = if *enabled { "CAPS ON" } else { "CAPS OFF" };
                let color = if *enabled { rgb(0x88c0d0) } else { rgb(0x4c566a) };
                ("⇪", text.to_string(), color)
            },
            OsdType::Volume(level) => {
                let icon = if *level == 0 { "🔇" } else if *level < 50 { "🔉" } else { "🔊" };
                (icon, self.volume_text.clone(), rgb(0x88c0d0))
            },
            _ => ("", "".to_string(), rgb(0x88c0d0))
        };

        div()
            .flex()
            .items_center()
            .justify_center()
            .w_64()
            .h_16()
            .bg(rgb(0x2e3440))
            .border_2()
            .border_color(color)
            .rounded_lg()
            .shadow_lg()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().text_xl().text_color(color).child(icon))
                    .child(div().text_sm().text_color(rgb(0xeceff4)).child(text))
            )
    }
}
