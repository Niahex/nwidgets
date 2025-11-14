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
        println!("[OSD] Creating OSD with type: {:?}", osd_type);
        
        let osd = Self {
            osd_type,
            visible: false,
            show_until: None,
            volume_text: String::new(),
        };

        // Subscribe to CapsLock changes
        cx.spawn(async move |this, cx| {
            let mut capslock_rx = CapsLockService::start_monitoring();
            println!("[OSD] ⌨️  Subscribed to CapsLock changes");

            while let Some(caps_on) = capslock_rx.recv().await {
                println!("[OSD] ⚠️  CAPS LOCK CHANGED: {}", caps_on);
                let result = this.update(cx, |osd, cx| {
                    osd.osd_type = OsdType::CapsLock(caps_on);
                    osd.visible = true;
                    osd.show_until = Some(Instant::now() + Duration::from_millis(2500));
                    println!("[OSD] 🔔 Calling cx.notify()");
                    cx.notify();
                });

                if let Err(e) = result {
                    println!("[OSD] ❌ Error updating OSD: {:?}", e);
                    break;
                }
            }
        }).detach();

        // Subscribe to NumLock changes
        cx.spawn(async move |this, cx| {
            let mut numlock_rx = NumLockService::start_monitoring();
            println!("[OSD] ⌨️  Subscribed to NumLock changes");

            while let Some(num_on) = numlock_rx.recv().await {
                println!("[OSD] ⚠️  NUM LOCK CHANGED: {}", num_on);
                let result = this.update(cx, |osd, cx| {
                    osd.osd_type = OsdType::NumLock(num_on);
                    osd.visible = true;
                    osd.show_until = Some(Instant::now() + Duration::from_millis(2500));
                    println!("[OSD] 🔔 Calling cx.notify()");
                    cx.notify();
                });

                if let Err(e) = result {
                    println!("[OSD] ❌ Error updating OSD: {:?}", e);
                    break;
                }
            }
        }).detach();

        // Subscribe to Volume changes
        cx.spawn(async move |this, cx| {
            let mut volume_rx = PipeWireService::start_monitoring();
            println!("[OSD] 🔊 Subscribed to Volume changes");

            while let Some((volume, _muted)) = volume_rx.recv().await {
                println!("[OSD] ⚠️  VOLUME CHANGED: {}%", volume);
                let result = this.update(cx, |osd, cx| {
                    osd.volume_text = format!("{}%", volume);
                    osd.osd_type = OsdType::Volume(volume);
                    osd.visible = true;
                    osd.show_until = Some(Instant::now() + Duration::from_millis(2500));
                    println!("[OSD] 🔔 Calling cx.notify()");
                    cx.notify();
                });

                if let Err(e) = result {
                    println!("[OSD] ❌ Error updating OSD: {:?}", e);
                    break;
                }
            }
        }).detach();

        // Timer to hide OSD after timeout
        cx.spawn(async move |this, cx| {
            println!("[OSD] ⏱️  Starting hide timer");
            loop {
                gpui::Timer::after(Duration::from_millis(100)).await;

                let result = this.update(cx, |osd, cx| {
                    if let Some(hide_time) = osd.show_until {
                        if Instant::now() >= hide_time && osd.visible {
                            println!("[OSD] ⏱️  Hiding OSD after timeout");
                            osd.visible = false;
                            osd.show_until = None;
                            cx.notify();
                        }
                    }
                });

                if let Err(e) = result {
                    println!("[OSD] ❌ Error in hide timer: {:?}", e);
                    break;
                }
            }
        }).detach();

        osd
    }
}

impl Render for Osd {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        println!("[OSD] 🎨 Render called - visible: {}, type: {:?}", self.visible, self.osd_type);

        if !self.visible {
            println!("[OSD] 🚫 Not visible, returning empty div");
            return div();
        }

        let (icon, text, color) = match &self.osd_type {
            OsdType::CapsLock(enabled) => {
                let text = if *enabled { "CAPS ON" } else { "CAPS OFF" };
                let color = if *enabled { rgb(0x88c0d0) } else { rgb(0x4c566a) };
                println!("[OSD] 🔑 Rendering caps lock: {} - {}", enabled, text);
                ("⇪", text.to_string(), color)
            },
            OsdType::NumLock(enabled) => {
                let text = if *enabled { "NUM ON" } else { "NUM OFF" };
                let color = if *enabled { rgb(0x88c0d0) } else { rgb(0x4c566a) };
                println!("[OSD] 🔢 Rendering num lock: {} - {}", enabled, text);
                ("⇭", text.to_string(), color)
            },
            OsdType::Volume(level) => {
                let icon = if *level == 0 { "🔇" } else if *level < 50 { "🔉" } else { "🔊" };
                println!("[OSD] 🔊 Rendering volume: {}% - {}", level, self.volume_text);
                (icon, self.volume_text.clone(), rgb(0x88c0d0))
            },
            OsdType::Microphone(enabled) => {
                let text = if *enabled { "MIC ON" } else { "MIC OFF" };
                let color = if *enabled { rgb(0x88c0d0) } else { rgb(0xbf616a) };
                println!("[OSD] 🎤 Rendering microphone: {} - {}", enabled, text);
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
