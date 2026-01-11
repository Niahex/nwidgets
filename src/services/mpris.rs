use futures_util::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use zbus::proxy;
use zbus::zvariant::OwnedValue;
use zbus::Connection;

#[derive(Clone, Debug, PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl From<String> for PlaybackStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Playing" => PlaybackStatus::Playing,
            "Paused" => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct MprisMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MprisPlayer {
    pub player_name: String,
    pub status: PlaybackStatus,
    pub metadata: MprisMetadata,
}

#[derive(Clone)]
pub struct MprisStateChanged;

pub struct MprisService {
    current_player: Arc<RwLock<Option<MprisPlayer>>>,
}

// D-Bus proxy for MPRIS2 Player interface
#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_service = "org.mpris.MediaPlayer2.spotify",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait MediaPlayer2Player {
    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;

    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
    fn play_pause(&self) -> zbus::Result<()>;
}

impl EventEmitter<MprisStateChanged> for MprisService {}

impl MprisService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let current_player = Arc::new(RwLock::new(None));
        let current_player_clone = Arc::clone(&current_player);

        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<Option<MprisPlayer>>();

        // 1. Worker Task (Tokio)
        gpui_tokio::Tokio::spawn(cx, async move { Self::mpris_worker(ui_tx).await }).detach();

        // 2. UI Task (GPUI)
        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(new_player) = ui_rx.next().await {
                    let state_changed = {
                        let mut current = current_player_clone.write();
                        let changed = *current != new_player;
                        if changed {
                            *current = new_player;
                        }
                        changed
                    };

                    if state_changed {
                        let _ = this.update(&mut cx, |_, cx| {
                            cx.emit(MprisStateChanged);
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();

        Self { current_player }
    }

    pub fn current_player(&self) -> Option<MprisPlayer> {
        self.current_player.read().clone()
    }

    pub fn play_pause(&self, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async {
            if let Ok(conn) = Connection::session().await {
                if let Ok(proxy) = MediaPlayer2PlayerProxy::new(&conn).await {
                    let _ = proxy.play_pause().await;
                }
            }
        })
        .detach();
    }

    pub fn next(&self, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async {
            if let Ok(conn) = Connection::session().await {
                if let Ok(proxy) = MediaPlayer2PlayerProxy::new(&conn).await {
                    let _ = proxy.next().await;
                }
            }
        })
        .detach();
    }

    pub fn previous(&self, cx: &mut Context<Self>) {
        gpui_tokio::Tokio::spawn(cx, async {
            if let Ok(conn) = Connection::session().await {
                if let Ok(proxy) = MediaPlayer2PlayerProxy::new(&conn).await {
                    let _ = proxy.previous().await;
                }
            }
        })
        .detach();
    }

    pub fn volume_up(&self) {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("playerctl")
                .args(["-p", "spotify", "volume", "0.05+"])
                .status();
        });
    }

    pub fn volume_down(&self) {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("playerctl")
                .args(["-p", "spotify", "volume", "0.05-"])
                .status();
        });
    }

    async fn mpris_worker(ui_tx: futures::channel::mpsc::UnboundedSender<Option<MprisPlayer>>) {
        loop {
            // Try to connect to Spotify via D-Bus
            let connection = match Connection::session().await {
                Ok(conn) => conn,
                Err(_) => {
                    // Connection failed, wait and retry
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            let proxy = match MediaPlayer2PlayerProxy::new(&connection).await {
                Ok(p) => p,
                Err(_) => {
                    // Spotify not running, set state to None
                    let _ = ui_tx.unbounded_send(None);
                    // Wait before retrying
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            // Get initial state
            if let Ok(player) = Self::fetch_player_state(&proxy).await {
                let _ = ui_tx.unbounded_send(Some(player));
            }

            // Subscribe to property changes
            let mut status_stream = proxy.receive_playback_status_changed().await;
            let mut metadata_stream = proxy.receive_metadata_changed().await;

            // Event loop: listen for D-Bus property changes
            loop {
                tokio::select! {
                    status_change = status_stream.next() => {
                        if status_change.is_none() { break; }
                        if let Ok(player) = Self::fetch_player_state(&proxy).await {
                            let _ = ui_tx.unbounded_send(Some(player));
                        }
                    }
                    metadata_change = metadata_stream.next() => {
                        if metadata_change.is_none() { break; }
                        if let Ok(player) = Self::fetch_player_state(&proxy).await {
                            let _ = ui_tx.unbounded_send(Some(player));
                        }
                    }
                }
            }

            // Connection lost (streams ended), wait before reconnecting
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    async fn fetch_player_state(proxy: &MediaPlayer2PlayerProxy<'_>) -> zbus::Result<MprisPlayer> {
        let status_str = proxy.playback_status().await?;
        let status = PlaybackStatus::from(status_str);

        let mut metadata = MprisMetadata::default();

        if let Ok(metadata_map) = proxy.metadata().await {
            // Extract title
            if let Some(value) = metadata_map.get("xesam:title") {
                if let Ok(title_str) = value.downcast_ref::<zbus::zvariant::Str>() {
                    metadata.title = Some(title_str.to_string());
                }
            }

            // Extract artist (it's an array)
            if let Some(value) = metadata_map.get("xesam:artist") {
                if let Ok(artist_array) = value.downcast_ref::<zbus::zvariant::Array>() {
                    if let Ok(Some(first_artist)) = artist_array.get::<zbus::zvariant::Value>(0) {
                        if let Ok(artist_str) = first_artist.downcast_ref::<zbus::zvariant::Str>() {
                            metadata.artist = Some(artist_str.to_string());
                        }
                    }
                }
            }

            // Extract album
            if let Some(value) = metadata_map.get("xesam:album") {
                if let Ok(album_str) = value.downcast_ref::<zbus::zvariant::Str>() {
                    metadata.album = Some(album_str.to_string());
                }
            }
        }

        Ok(MprisPlayer {
            player_name: "spotify".to_string(),
            status,
            metadata,
        })
    }
}

// Global accessor
struct GlobalMprisService(Entity<MprisService>);
impl Global for GlobalMprisService {}

impl MprisService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalMprisService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalMprisService(service.clone()));
        service
    }
}
