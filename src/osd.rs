use gpui::*;
use std::process::Command;
use std::time::Duration;

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
}

impl Osd {
    pub fn new(osd_type: OsdType) -> Self {
        Self {
            osd_type,
            visible: false,
        }
    }

    pub fn show_caps_lock(&mut self, cx: &mut Context<Self>) {
        let caps_on = Self::get_caps_lock_state();
        self.osd_type = OsdType::CapsLock(caps_on);
        self.show_and_hide(cx);
    }

    pub fn show_volume(&mut self, cx: &mut Context<Self>) {
        let volume = Self::get_volume_level();
        self.osd_type = OsdType::Volume(volume);
        self.show_and_hide(cx);
    }

    fn show_and_hide(&mut self, cx: &mut Context<Self>) {
        self.visible = true;
        cx.notify();
        
        cx.spawn(|this, mut cx| async move {
            cx.background_executor().timer(Duration::from_secs(2)).await;
            _ = this.update(&mut cx, |osd, cx| {
                osd.visible = false;
                cx.notify();
            });
        }).detach();
    }

    fn get_caps_lock_state() -> bool {
        Command::new("xset")
            .args(&["q"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .contains("Caps Lock:   on")
            })
            .unwrap_or(false)
    }

    fn get_volume_level() -> u8 {
        Command::new("pactl")
            .args(&["get-sink-volume", "@DEFAULT_SINK@"])
            .output()
            .and_then(|output| {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str
                    .split_whitespace()
                    .find(|s| s.ends_with('%'))
                    .and_then(|s| s.trim_end_matches('%').parse().ok())
                    .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Parse error"))
            })
            .unwrap_or(50)
    }

    pub fn start_monitoring(&mut self, cx: &mut Context<Self>) {
        cx.spawn(|this, mut cx| async move {
            let mut last_caps = false;
            let mut last_volume = 50u8;
            
            loop {
                cx.background_executor().timer(Duration::from_millis(100)).await;
                
                let caps = Self::get_caps_lock_state();
                let volume = Self::get_volume_level();
                
                if caps != last_caps {
                    _ = this.update(&mut cx, |osd, cx| osd.show_caps_lock(cx));
                    last_caps = caps;
                }
                
                if volume != last_volume {
                    _ = this.update(&mut cx, |osd, cx| osd.show_volume(cx));
                    last_volume = volume;
                }
            }
        }).detach();
    }
}

impl Render for Osd {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div();
        }

        let (icon, text) = match &self.osd_type {
            OsdType::CapsLock(enabled) => {
                ("⇪", if *enabled { "CAPS ON" } else { "CAPS OFF" })
            },
            OsdType::NumLock(enabled) => {
                ("123", if *enabled { "NUM ON" } else { "NUM OFF" })
            },
            OsdType::Volume(level) => {
                let icon = if *level == 0 { "🔇" } else if *level < 50 { "🔉" } else { "🔊" };
                (icon, "VOLUME")
            },
            OsdType::Microphone(muted) => {
                ("🎤", if *muted { "MIC MUTED" } else { "MIC ON" })
            },
        };

        let color = match &self.osd_type {
            OsdType::CapsLock(enabled) | OsdType::NumLock(enabled) => {
                if *enabled { rgb(0x88c0d0) } else { rgb(0x4c566a) }
            },
            OsdType::Volume(_) => rgb(0x88c0d0),
            OsdType::Microphone(muted) => {
                if *muted { rgb(0xbf616a) } else { rgb(0xa3be8c) }
            },
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
