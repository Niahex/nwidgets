use futures_util::StreamExt;
use gpui::prelude::*;
use gpui::{App, AsyncApp, BackgroundExecutor, Context, Entity, EventEmitter, Global, WeakEntity};
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
    executor: BackgroundExecutor,
    spotify_running: Arc<RwLock<bool>>,
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
        let spotify_running = Arc::new(RwLock::new(false));
        let spotify_running_clone = Arc::clone(&spotify_running);

        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded::<Option<MprisPlayer>>();

        // Subscribe to Hyprland window changes to detect Spotify
        let hyprland = crate::services::hyprland::HyprlandService::global(cx);
        let spotify_running_for_sub = Arc::clone(&spotify_running);
        cx.subscribe(&hyprland, move |_, hyprland, _: &crate::services::hyprland::ActiveWindowChanged, _cx| {
            let is_spotify = hyprland.read(_cx).is_window_open("spotify");
            
            let mut running = spotify_running_for_sub.write();
            if *running != is_spotify {
                *running = is_spotify;
                eprintln!("[MPRIS] Spotify window state: {}", is_spotify);
            }
        }).detach();

        // 1. Worker Task (Tokio)
        gpui_tokio::Tokio::spawn(cx, async move { 
            Self::mpris_worker(ui_tx, spotify_running_clone).await 
        }).detach();

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

        Self {
            current_player,
            executor: cx.background_executor().clone(),
            spotify_running,
        }
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
        self.executor
            .spawn(async {
                let _ = std::process::Command::new("playerctl")
                    .args(["-p", "spotify", "volume", "0.05+"])
                    .status();
            })
            .detach();
    }

    pub fn volume_down(&self) {
        self.executor
            .spawn(async {
                let _ = std::process::Command::new("playerctl")
                    .args(["-p", "spotify", "volume", "0.05-"])
                    .status();
            })
            .detach();
    }

    async fn mpris_worker(
        ui_tx: futures::channel::mpsc::UnboundedSender<Option<MprisPlayer>>,
        spotify_running: Arc<RwLock<bool>>,
    ) {
        loop {
            // Wait for Spotify window to open
            while !*spotify_running.read() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            eprintln!("[MPRIS] Spotify window detected, connecting to DBus...");

            // Try to connect once
            let connection = match Connection::session().await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("[MPRIS] Failed to connect to session bus: {}", e);
                    continue;
                }
            };

            let proxy = match MediaPlayer2PlayerProxy::new(&connection).await {
                Ok(p) => {
                    eprintln!("[MPRIS] Connected to Spotify MPRIS");
                    p
                }
                Err(e) => {
                    eprintln!("[MPRIS] Spotify MPRIS not available: {} - waiting for window close", e);
                    // DBus service doesn't exist, wait for window to close then retry
                    while *spotify_running.read() {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                    let _ = ui_tx.unbounded_send(None);
                    continue;
                }
            };

            // Get initial state
            match Self::fetch_player_state(&proxy).await {
                Ok(player) => {
                    eprintln!("[MPRIS] Initial state: {:?}", player.metadata.title);
                    let _ = ui_tx.unbounded_send(Some(player));
                }
                Err(e) => {
                    eprintln!("[MPRIS] Failed to fetch initial state: {}", e);
                    let _ = ui_tx.unbounded_send(None);
                    continue;
                }
            }

            // Subscribe to property changes
            let mut status_stream = proxy.receive_playback_status_changed().await;
            let mut metadata_stream = proxy.receive_metadata_changed().await;

            // Event loop: listen for D-Bus property changes or Spotify closing
            loop {
                tokio::select! {
                    status_change = status_stream.next() => {
                        if status_change.is_none() {
                            eprintln!("[MPRIS] Status stream ended");
                            break;
                        }
                        if let Ok(player) = Self::fetch_player_state(&proxy).await {
                            let _ = ui_tx.unbounded_send(Some(player));
                        }
                    }
                    metadata_change = metadata_stream.next() => {
                        if metadata_change.is_none() {
                            eprintln!("[MPRIS] Metadata stream ended");
                            break;
                        }
                        if let Ok(player) = Self::fetch_player_state(&proxy).await {
                            let _ = ui_tx.unbounded_send(Some(player));
                        }
                    }
                    _ = async {
                        while *spotify_running.read() {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    } => {
                        eprintln!("[MPRIS] Spotify window closed");
                        break;
                    }
                }
            }

            // Spotify closed, clear state
            eprintln!("[MPRIS] Clearing MPRIS state...");
            let _ = ui_tx.unbounded_send(None);
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
