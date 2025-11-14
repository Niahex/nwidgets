use gpui::*;
use std::fs;
use std::time::{Duration, Instant};
use std::process::Command;

#[derive(Clone, Debug)]
pub enum OsdType {
    CapsLock(bool),
    NumLock(bool),
    Volume(u8),
    Microphone(bool),
}

pub struct Osd {
    osd_type: OsdType,
    visible: bool,
    show_until: Option<Instant>,
    volume_text: String,
}

impl Osd {
    pub fn new(osd_type: OsdType, cx: &mut Context<Self>) -> Self {
        println!("[OSD] Creating OSD with type: {:?}", osd_type);
        
        let osd = Self {
            osd_type,
            visible: false,
            show_until: None,
            volume_text: String::new(),
        };

        // Monitor caps lock and volume changes
        cx.spawn(async move |this, cx| {
            let mut last_caps_state = Self::get_caps_lock_state();
            let mut last_volume = Self::get_volume_level();
            println!("[OSD] Starting monitoring - caps: {}, volume: {}", last_caps_state, last_volume);
            
            loop {
                Timer::after(Duration::from_millis(100)).await;
                
                let caps_on = Self::get_caps_lock_state();
                let volume = Self::get_volume_level();
                let now = Instant::now();
                
                let _ = this.update(cx, |osd, cx| {
                    let mut should_notify = false;
                    
                    // Check caps lock change
                    if caps_on != last_caps_state {
                        println!("[OSD] Caps lock changed: {} -> {}", last_caps_state, caps_on);
                        osd.osd_type = OsdType::CapsLock(caps_on);
                        osd.visible = true;
                        osd.show_until = Some(now + Duration::from_millis(2500));
                        should_notify = true;
                    }
                    
                    // Check volume change
                    if volume != last_volume {
                        println!("[OSD] Volume changed: {} -> {}", last_volume, volume);
                        osd.volume_text = format!("{}%", volume);
                        osd.osd_type = OsdType::Volume(volume);
                        osd.visible = true;
                        osd.show_until = Some(now + Duration::from_millis(2500));
                        should_notify = true;
                    }
                    
                    // Hide after timeout
                    if let Some(hide_time) = osd.show_until {
                        if now >= hide_time && osd.visible {
                            println!("[OSD] Hiding OSD after timeout");
                            osd.visible = false;
                            osd.show_until = None;
                            should_notify = true;
                        }
                    }
                    
                    if should_notify {
                        println!("[OSD] Notifying render - visible: {}", osd.visible);
                        cx.notify();
                    }
                });
                
                last_caps_state = caps_on;
                last_volume = volume;
            }
        }).detach();

        osd
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
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        println!("[OSD] Rendering - visible: {}", self.visible);
        
        if !self.visible {
            return div();
        }

        let (icon, text, color) = match &self.osd_type {
            OsdType::CapsLock(enabled) => {
                let text = if *enabled { "CAPS ON" } else { "CAPS OFF" };
                let color = if *enabled { rgb(0x88c0d0) } else { rgb(0x4c566a) };
                println!("[OSD] Rendering caps lock: {} - {}", enabled, text);
                ("⇪", text.to_string(), color)
            },
            OsdType::Volume(level) => {
                let icon = if *level == 0 { "🔇" } else if *level < 50 { "🔉" } else { "🔊" };
                println!("[OSD] Rendering volume: {}% - {}", level, self.volume_text);
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
