use gpui::*;
use std::time::{Duration, Instant};
use crate::services::{CapsLockService, NumLockService, PipeWireService};

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
        let osd = Self {
            osd_type,
            visible: false,
            show_until: None,
            volume_text: String::new(),
        };

        // Monitor CapsLock, NumLock and Volume changes
        cx.spawn(async move |this, cx| {
            let capslock_service = CapsLockService::new();
            let numlock_service = NumLockService::new();
            let pipewire_service = PipeWireService::new();

            let mut last_caps = capslock_service.is_enabled();
            let mut last_num = numlock_service.is_enabled();
            let mut last_volume = pipewire_service.get_volume();

            loop {
                gpui::Timer::after(Duration::from_millis(100)).await;

                let caps_on = capslock_service.is_enabled();
                let num_on = numlock_service.is_enabled();
                let volume = pipewire_service.get_volume();

                let _ = this.update(cx, |osd, cx| {
                    // Check CapsLock change
                    if caps_on != last_caps {
                        println!("[OSD] ⇪ CapsLock: {}", if caps_on { "ON" } else { "OFF" });
                        osd.osd_type = OsdType::CapsLock(caps_on);
                        osd.visible = true;
                        osd.show_until = Some(Instant::now() + Duration::from_millis(2500));
                        cx.notify();
                    }

                    // Check NumLock change
                    if num_on != last_num {
                        println!("[OSD] ⇭ NumLock: {}", if num_on { "ON" } else { "OFF" });
                        osd.osd_type = OsdType::NumLock(num_on);
                        osd.visible = true;
                        osd.show_until = Some(Instant::now() + Duration::from_millis(2500));
                        cx.notify();
                    }

                    // Check volume change
                    if volume != last_volume {
                        println!("[OSD] 🔊 Volume: {}%", volume);
                        osd.volume_text = format!("{}%", volume);
                        osd.osd_type = OsdType::Volume(volume);
                        osd.visible = true;
                        osd.show_until = Some(Instant::now() + Duration::from_millis(2500));
                        cx.notify();
                    }
                });

                last_caps = caps_on;
                last_num = num_on;
                last_volume = volume;
            }
        }).detach();

        // Timer to hide OSD after timeout
        cx.spawn(async move |this, cx| {
            loop {
                gpui::Timer::after(Duration::from_millis(100)).await;
                let _ = this.update(cx, |osd, cx| {
                    if let Some(hide_time) = osd.show_until {
                        if Instant::now() >= hide_time && osd.visible {
                            osd.visible = false;
                            osd.show_until = None;
                            cx.notify();
                        }
                    }
                });
            }
        }).detach();

        osd
    }
}

impl Render for Osd {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div();
        }

        let (icon, text, color) = match &self.osd_type {
            OsdType::CapsLock(enabled) => {
                let text = if *enabled { "CAPS ON" } else { "CAPS OFF" };
                let color = if *enabled { rgb(0x88c0d0) } else { rgb(0x4c566a) };
                ("⇪", text.to_string(), color)
            },
            OsdType::NumLock(enabled) => {
                let text = if *enabled { "NUM ON" } else { "NUM OFF" };
                let color = if *enabled { rgb(0x88c0d0) } else { rgb(0x4c566a) };
                ("⇭", text.to_string(), color)
            },
            OsdType::Volume(level) => {
                let icon = if *level == 0 { "🔇" } else if *level < 50 { "🔉" } else { "🔊" };
                (icon, self.volume_text.clone(), rgb(0x88c0d0))
            },
            OsdType::Microphone(enabled) => {
                let text = if *enabled { "MIC ON" } else { "MIC OFF" };
                let color = if *enabled { rgb(0x88c0d0) } else { rgb(0xbf616a) };
                ("🎤", text.to_string(), color)
            }
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
