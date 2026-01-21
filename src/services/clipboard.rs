use crate::services::hyprland::HyprlandService;
use futures::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Entity, EventEmitter};
use std::collections::VecDeque;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Clone)]
pub struct ClipboardEvent {
    #[allow(dead_code)]
    pub content: String,
}

pub struct ClipboardMonitor {
    history: VecDeque<String>,
    last_content: Option<String>,
}

impl EventEmitter<ClipboardEvent> for ClipboardMonitor {}

impl ClipboardMonitor {
    pub fn init(cx: &mut App) -> Entity<Self> {
        let model = cx.new(|_cx| Self {
            history: VecDeque::with_capacity(50),
            last_content: None,
        });
        let weak_model = model.downgrade();

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<String>();

        let hyprland_service = HyprlandService::global(cx);

        // 1. Worker Task (Tokio): Watcher asynchrone
        gpui_tokio::Tokio::spawn(cx, async move {
            let child = tokio::process::Command::new("wl-paste")
                .args(["--watch", "echo", "clipboard_changed"])
                .stdout(Stdio::piped())
                .spawn();

            match child {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let mut reader = BufReader::new(stdout).lines();
                        while let Ok(Some(_)) = reader.next_line().await {
                            // Récupérer le contenu actuel du clipboard
                            if let Ok(output) = tokio::process::Command::new("wl-paste")
                                .arg("-n")
                                .output()
                                .await
                            {
                                if let Ok(content) = String::from_utf8(output.stdout) {
                                    if !content.trim().is_empty()
                                        && tx.unbounded_send(content).is_err()
                                    {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    let _ = child.kill().await;
                }
                Err(e) => {
                    log::error!("Failed to start wl-paste watcher: {e}");
                }
            }
        })
        .detach();

        // 2. UI Task (GPUI): Réception des événements
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let weak_model = weak_model.clone();
            async move {
                while let Some(content) = rx.next().await {
                    let _ = weak_model.update(&mut cx, |this, cx| {
                        let should_exclude = hyprland_service
                            .read(cx)
                            .active_window()
                            .as_ref()
                            .map(|w| w.class == "org.keepassxc.KeePassXC")
                            .unwrap_or(false);

                        if !should_exclude && this.last_content.as_ref() != Some(&content) {
                            this.last_content = Some(content.clone());
                            this.history.push_front(content.clone());
                            if this.history.len() > 50 {
                                this.history.pop_back();
                            }
                            cx.emit(ClipboardEvent { content });
                        }
                    });
                }
            }
        })
        .detach();

        model
    }

    pub fn get_history(&self) -> Vec<String> {
        self.history.iter().cloned().collect()
    }
}
