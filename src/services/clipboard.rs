use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Entity, EventEmitter};
use std::io::BufRead;

#[derive(Clone)]
pub struct ClipboardEvent;

pub struct ClipboardMonitor;

impl EventEmitter<ClipboardEvent> for ClipboardMonitor {}

impl ClipboardMonitor {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let model = cx.new(|_cx| Self);
        let weak_model = model.downgrade();

        // 1. Créer un channel pour communiquer entre le thread watcher et GPUI
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<()>();

        // 2. Lancer un thread dédié pour wl-paste --watch (bloquant)
        std::thread::spawn(move || {
            let child = std::process::Command::new("wl-paste")
                .args(["--watch", "echo", "changed"])
                .stdout(std::process::Stdio::piped())
                .spawn();

            match child {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let reader = std::io::BufReader::new(stdout);
                        for _line in reader.lines() {
                            if tx.unbounded_send(()).is_err() {
                                break;
                            }
                        }
                    }
                    let _ = child.kill();
                }
                Err(e) => {
                    eprintln!("[Clipboard] Failed to start wl-paste watcher: {e}");
                }
            }
        });

        // 3. Consommer les événements sur le thread GPUI
        cx.spawn(|cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while rx.next().await.is_some() {
                    let _ = weak_model.update(&mut cx, |_this, cx| {
                        cx.emit(ClipboardEvent);
                    });
                }
            }
        })
        .detach();

        model
    }
}
