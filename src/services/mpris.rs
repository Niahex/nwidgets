use gpui::prelude::*;
use gpui::{App, AsyncApp, Context, Entity, EventEmitter, Global, WeakEntity};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, PartialEq)]
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
pub struct MprisStateChanged {
    pub player: Option<MprisPlayer>,
}

pub struct MprisService {
    current_player: Arc<RwLock<Option<MprisPlayer>>>,
}

impl EventEmitter<MprisStateChanged> for MprisService {}

impl MprisService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let current_player = Arc::new(RwLock::new(Self::fetch_mpris_state()));

        let current_player_clone = Arc::clone(&current_player);

        // Poll MPRIS state periodically
        cx.spawn(async move |this, mut cx| {
            Self::monitor_mpris(this, current_player_clone, &mut cx).await
        })
        .detach();

        Self { current_player }
    }

    pub fn current_player(&self) -> Option<MprisPlayer> {
        self.current_player.read().clone()
    }

    pub fn play_pause(&self) {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("playerctl")
                .args(["play-pause"])
                .status();
        });
    }

    pub fn next(&self) {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("playerctl")
                .args(["next"])
                .status();
        });
    }

    pub fn previous(&self) {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("playerctl")
                .args(["previous"])
                .status();
        });
    }

    async fn monitor_mpris(
        this: WeakEntity<Self>,
        current_player: Arc<RwLock<Option<MprisPlayer>>>,
        cx: &mut AsyncApp,
    ) {
        loop {
            cx.background_executor()
                .timer(Duration::from_secs(2))
                .await;

            let new_player = Self::fetch_mpris_state();

            let state_changed = {
                let mut current = current_player.write();
                let changed = *current != new_player;
                if changed {
                    *current = new_player.clone();
                }
                changed
            };

            if state_changed {
                if let Ok(()) = this.update(cx, |_, cx| {
                    cx.emit(MprisStateChanged { player: new_player });
                    cx.notify();
                }) {}
            }
        }
    }

    fn fetch_mpris_state() -> Option<MprisPlayer> {
        // Check if playerctl is available and has a player
        let status_output = std::process::Command::new("playerctl")
            .args(["status"])
            .output()
            .ok()?;

        if !status_output.status.success() {
            return None;
        }

        let status_str = String::from_utf8(status_output.stdout).ok()?;
        let status = match status_str.trim() {
            "Playing" => PlaybackStatus::Playing,
            "Paused" => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        };

        // Get player name
        let player_name = std::process::Command::new("playerctl")
            .args(["-l"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().next().map(|l| l.to_string()))
            .unwrap_or_else(|| "Unknown".to_string());

        // Get metadata
        let title = std::process::Command::new("playerctl")
            .args(["metadata", "title"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        let artist = std::process::Command::new("playerctl")
            .args(["metadata", "artist"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        let album = std::process::Command::new("playerctl")
            .args(["metadata", "album"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        Some(MprisPlayer {
            player_name,
            status,
            metadata: MprisMetadata {
                title,
                artist,
                album,
            },
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
        let service = cx.new(|cx| Self::new(cx));
        cx.set_global(GlobalMprisService(service.clone()));
        service
    }
}
