use gpui::*;
use std::fs;
use std::time::{Duration, Instant};

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
    show_until: Option<Instant>,
}

impl Osd {
    pub fn new(osd_type: OsdType) -> Self {
        Self {
            osd_type,
            visible: false,
            last_caps_state: false,
            show_until: None,
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
}

impl Render for Osd {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let caps_on = Self::get_caps_lock_state();
        let now = Instant::now();
        
        // Show OSD when caps lock state changes
        if caps_on != self.last_caps_state {
            self.last_caps_state = caps_on;
            self.visible = true;
            self.show_until = Some(now + Duration::from_millis(2500));
            self.osd_type = OsdType::CapsLock(caps_on);
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

        let text = if caps_on { "CAPS ON" } else { "CAPS OFF" };
        let color = if caps_on { rgb(0x88c0d0) } else { rgb(0x4c566a) };

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
                    .child(div().text_xl().text_color(color).child("⇪"))
                    .child(div().text_sm().text_color(rgb(0xeceff4)).child(text))
            )
    }
}
