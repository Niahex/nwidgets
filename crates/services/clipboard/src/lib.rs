use chrono::{DateTime, Local};
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Entity, EventEmitter, Global};
use std::collections::VecDeque;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: DateTime<Local>,
}

#[derive(Clone)]
pub struct ClipboardChanged;

pub struct ClipboardService {
    pub history: VecDeque<ClipboardEntry>,
    last_content: Option<String>,
}

impl EventEmitter<ClipboardChanged> for ClipboardService {}

struct GlobalClipboardService(Entity<ClipboardService>);
impl Global for GlobalClipboardService {}

impl ClipboardService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalClipboardService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|_cx| Self {
            history: VecDeque::with_capacity(50),
            last_content: None,
        });
        cx.set_global(GlobalClipboardService(service.clone()));

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<String>();

        // Watcher async via wl-paste --watch
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
                            let timeout = std::time::Duration::from_secs(2);
                            let result = tokio::time::timeout(timeout, async {
                                tokio::process::Command::new("wl-paste")
                                    .arg("-n")
                                    .output()
                                    .await
                            })
                            .await;

                            if let Ok(Ok(output)) = result {
                                if let Ok(content) = String::from_utf8(output.stdout) {
                                    let content = content.trim().to_string();
                                    if !content.is_empty() && tx.unbounded_send(content).is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        let _ = child.kill().await;
                    }
                }
                Err(e) => log::error!("Failed to start wl-paste watcher: {e}"),
            }
        })
        .detach();

        // Réception des entrées clipboard dans le thread UI
        let weak = service.downgrade();
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(content) = rx.next().await {
                    let _ = weak.update(&mut cx, |this, cx| {
                        if this.last_content.as_deref() != Some(&content) {
                            this.last_content = Some(content.clone());
                            this.history.push_front(ClipboardEntry {
                                content,
                                timestamp: Local::now(),
                            });
                            if this.history.len() > 50 {
                                this.history.pop_back();
                            }
                            cx.emit(ClipboardChanged);
                            cx.notify();
                        }
                    });
                }
            }
        })
        .detach();

        service
    }

    pub fn search(&self, query: &str) -> Vec<ClipboardEntry> {
        // "clip" seul → tout l'historique
        // "clip <term>" → filtré par contenu
        let term = query.strip_prefix("clip").unwrap_or("").trim().to_lowercase();
        if term.is_empty() {
            return self.history.iter().cloned().collect();
        }
        self.history
            .iter()
            .filter(|e| e.content.to_lowercase().contains(&term))
            .cloned()
            .collect()
    }

    pub fn copy_to_clipboard(content: &str) {
        let content = content.to_string();
        std::thread::spawn(move || {
            let _ = std::process::Command::new("wl-copy")
                .arg(&content)
                .status();
        });
    }
}
