use gpui::prelude::*;
use gpui::{App, Entity, EventEmitter, AsyncApp};
use std::process::Command;
use std::time::Duration;

#[derive(Clone)]
pub struct ClipboardEvent;

pub struct ClipboardMonitor;

impl EventEmitter<ClipboardEvent> for ClipboardMonitor {}

impl ClipboardMonitor {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let model = cx.new(|_cx| Self);
        let weak_model = model.downgrade();

        cx.spawn(|cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let mut last_content = Self::get_clipboard_content();

                loop {
                    cx.background_executor().timer(Duration::from_millis(250)).await;
                    
                    let current_content = Self::get_clipboard_content();

                    // Si le contenu a changé et n'est pas vide (et différent de None)
                    if current_content != last_content && current_content.is_some() {
                        last_content = current_content;

                        let _ = weak_model.update(&mut cx, |_this, cx| {
                            cx.emit(ClipboardEvent);
                        });
                    }
                }
            }
        }).detach();

        model
    }

    fn get_clipboard_content() -> Option<String> {
        Command::new("wl-paste")
            .arg("-n") // No newline
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
    }
}
