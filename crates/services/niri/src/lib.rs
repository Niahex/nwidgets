use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global};
use serde::Deserialize;
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActiveWindow {
    pub title: String,
    pub app_id: String,
}

#[derive(Debug, Clone)]
pub struct ActiveWindowChanged(pub ActiveWindow);

pub struct NiriActiveWindowService {
    pub active_window: ActiveWindow,
}

impl EventEmitter<ActiveWindowChanged> for NiriActiveWindowService {}

struct GlobalNiriActiveWindowService(Entity<NiriActiveWindowService>);
impl Global for GlobalNiriActiveWindowService {}

#[derive(Deserialize)]
struct NiriResponse {
    #[serde(rename = "Ok")]
    ok: Option<NiriResponseOk>,
}

#[derive(Deserialize)]
struct NiriResponseOk {
    #[serde(rename = "FocusedWindow")]
    focused_window: Option<NiriWindowInfo>,
}

#[derive(Deserialize)]
struct NiriWindowInfo {
    title: Option<String>,
    app_id: Option<String>,
}

#[derive(Deserialize)]
struct NiriEventItem {
    #[serde(rename = "WindowsChanged")]
    windows_changed: Option<NiriWindowsChanged>,
    #[serde(rename = "WindowOpenedOrChanged")]
    window_opened_or_changed: Option<NiriWindowOpenedOrChanged>,
    #[serde(rename = "WindowFocusChanged")]
    window_focus_changed: Option<NiriWindowFocusChanged>,
}

#[derive(Deserialize)]
struct NiriWindowFocusChanged {
    id: Option<u64>,
}

#[derive(Deserialize)]
struct NiriWindowsChanged {
    windows: Vec<NiriWindowItem>,
}

#[derive(Deserialize)]
struct NiriWindowItem {
    title: Option<String>,
    app_id: Option<String>,
    is_focused: bool,
}

#[derive(Deserialize)]
struct NiriWindowOpenedOrChanged {
    window: NiriWindowItem,
}

impl NiriActiveWindowService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalNiriActiveWindowService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|_cx| Self {
            active_window: ActiveWindow::default(),
        });

        cx.set_global(GlobalNiriActiveWindowService(service.clone()));

        let (tx, mut rx) = mpsc::unbounded::<ActiveWindow>();

        // Background Tokio task reading Niri Unix socket
        gpui_tokio::Tokio::spawn(cx, async move {
            let socket_path = match env::var("NIRI_SOCKET") {
                Ok(path) => path,
                Err(_) => return,
            };

            async fn get_focused(path: &str) -> Option<ActiveWindow> {
                if let Ok(mut stream) = UnixStream::connect(path).await {
                    if stream.write_all(b"\"FocusedWindow\"\n").await.is_ok() {
                        let mut reader = BufReader::new(stream);
                        let mut line = String::new();
                        if reader.read_line(&mut line).await.is_ok() {
                            if let Ok(resp) = serde_json::from_str::<NiriResponse>(&line) {
                                if let Some(ok) = resp.ok {
                                    if let Some(info) = ok.focused_window {
                                        return Some(ActiveWindow {
                                            title: info.title.unwrap_or_default(),
                                            app_id: info.app_id.unwrap_or_default(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                None
            }

            // 1. Initial focused window
            if let Some(active) = get_focused(&socket_path).await {
                let _ = tx.unbounded_send(active);
            }

            // 2. Event stream
            if let Ok(mut stream) = UnixStream::connect(&socket_path).await {
                if stream.write_all(b"\"EventStream\"\n").await.is_ok() {
                    let mut lines = BufReader::new(stream).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if line.trim().is_empty() {
                            continue;
                        }

                        let mut new_active: Option<ActiveWindow> = None;

                        if let Ok(item) = serde_json::from_str::<NiriEventItem>(&line) {
                            if let Some(wc) = item.windows_changed {
                                if let Some(w) = wc.windows.into_iter().find(|w| w.is_focused) {
                                    new_active = Some(ActiveWindow {
                                        title: w.title.unwrap_or_default(),
                                        app_id: w.app_id.unwrap_or_default(),
                                    });
                                }
                            } else if let Some(woc) = item.window_opened_or_changed {
                                if woc.window.is_focused {
                                    new_active = Some(ActiveWindow {
                                        title: woc.window.title.unwrap_or_default(),
                                        app_id: woc.window.app_id.unwrap_or_default(),
                                    });
                                }
                            } else if item.window_focus_changed.is_some() {
                                new_active = get_focused(&socket_path).await;
                            }
                        }

                        if new_active.is_none() {
                            new_active = get_focused(&socket_path).await;
                        }

                        if let Some(active) = new_active {
                            let _ = tx.unbounded_send(active);
                        }
                    }
                }
            }
        })
        .detach();

        // UI handler reading updates from MPSC channel
        let service_entity = service.clone();
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(active) = rx.next().await {
                    let _ = cx.update(|cx| {
                        service_entity.update(cx, |srv, cx| {
                            if srv.active_window != active {
                                srv.active_window = active.clone();
                                cx.emit(ActiveWindowChanged(active));
                                cx.notify();
                            }
                        });
                    });
                }
            }
        })
        .detach();

        service
    }
}
