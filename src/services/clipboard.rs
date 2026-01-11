use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Entity, EventEmitter, WeakEntity};
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;

#[derive(Clone)]
pub struct ClipboardEvent;

pub struct ClipboardMonitor;

impl EventEmitter<ClipboardEvent> for ClipboardMonitor {}

impl ClipboardMonitor {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let model = cx.new(|_cx| Self);
        let weak_model = model.downgrade();

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<()>();

        // 1. Worker Task (Tokio): Watcher asynchrone
        gpui_tokio::Tokio::spawn(cx, async move {
            let child = tokio::process::Command::new("wl-paste")
                .args(["--watch", "echo", "changed"])
                .stdout(Stdio::piped())
                .spawn();

            match child {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let mut reader = BufReader::new(stdout).lines();
                        while let Ok(Some(_line)) = reader.next_line().await {
                            if tx.unbounded_send(()).is_err() {
                                break;
                            }
                        }
                    }
                    let _ = child.kill().await;
                }
                Err(e) => {
                    eprintln!("[Clipboard] Failed to start wl-paste watcher: {e}");
                }
            }
        }).detach();

        // 2. UI Task (GPUI): Réception des événements
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let weak_model = weak_model.clone();
            async move {
                while rx.next().await.is_some() {
                    let _ = weak_model.update(&mut cx, |_, cx| {
                        cx.emit(ClipboardEvent);
                    });
                }
            }
        })
        .detach();

        model
    }
}