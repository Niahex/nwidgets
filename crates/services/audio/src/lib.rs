use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Context, Entity, EventEmitter, Global};
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDevice {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AudioState {
    pub sink_volume: u8,
    pub sink_muted: bool,
    pub sink_name: String,
    pub source_volume: u8,
    pub source_muted: bool,
    pub source_name: String,
    pub sinks: Vec<AudioDevice>,
    pub sources: Vec<AudioDevice>,
}

#[derive(Debug, Clone)]
pub struct AudioStateChanged;

pub struct AudioService {
    pub state: AudioState,
}

impl EventEmitter<AudioStateChanged> for AudioService {}

struct GlobalAudioService(Entity<AudioService>);
impl Global for GlobalAudioService {}

impl AudioService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalAudioService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|_cx| Self {
            state: AudioState::default(),
        });

        cx.set_global(GlobalAudioService(service.clone()));

        let (tx, mut rx) = mpsc::unbounded::<AudioState>();

        // Background worker to monitor Pipewire / wpctl / pactl events
        gpui_tokio::Tokio::spawn(cx, async move {
            let fetch_state = || async {
                let mut state = AudioState::default();

                // Get Sink Volume
                if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output().await {
                    let s = String::from_utf8_lossy(&out.stdout);
                    if let Some(vol_str) = s.split_whitespace().nth(1) {
                        if let Ok(vol) = vol_str.parse::<f32>() {
                            state.sink_volume = (vol * 100.0).round() as u8;
                        }
                    }
                    state.sink_muted = s.contains("[MUTED]");
                }

                // Get Source Volume
                if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"]).output().await {
                    let s = String::from_utf8_lossy(&out.stdout);
                    if let Some(vol_str) = s.split_whitespace().nth(1) {
                        if let Ok(vol) = vol_str.parse::<f32>() {
                            state.source_volume = (vol * 100.0).round() as u8;
                        }
                    }
                    state.source_muted = s.contains("[MUTED]");
                }

                state
            };

            // Send initial audio state
            let mut last_state = fetch_state().await;
            let _ = tx.unbounded_send(last_state.clone());

            // Polling loop (150ms) to guarantee volume updates across Pipewire/ALSA/NixOS
            loop {
                tokio::time::sleep(Duration::from_millis(150)).await;
                let current_state = fetch_state().await;
                if current_state != last_state {
                    last_state = current_state.clone();
                    if tx.unbounded_send(current_state).is_err() {
                        break;
                    }
                }
            }
        })
        .detach();

        // UI Thread listener
        let service_entity = service.clone();
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Some(new_state) = rx.next().await {
                    let _ = cx.update(|cx| {
                        service_entity.update(cx, |srv, cx| {
                            if srv.state != new_state {
                                srv.state = new_state;
                                cx.emit(AudioStateChanged);
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

    pub fn set_sink_volume(&mut self, volume_percent: u8, cx: &mut Context<Self>) {
        if self.state.sink_volume != volume_percent {
            self.state.sink_volume = volume_percent;
            cx.emit(AudioStateChanged);
            cx.notify();
        }
        let vol_str = format!("{}%", volume_percent);
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &vol_str])
                .status()
                .await;
        })
        .detach();
    }

    pub fn set_source_volume(&mut self, volume_percent: u8, cx: &mut Context<Self>) {
        if self.state.source_volume != volume_percent {
            self.state.source_volume = volume_percent;
            cx.emit(AudioStateChanged);
            cx.notify();
        }
        let vol_str = format!("{}%", volume_percent);
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SOURCE@", &vol_str])
                .status()
                .await;
        })
        .detach();
    }

    pub fn toggle_sink_mute(&mut self, cx: &mut Context<Self>) {
        self.state.sink_muted = !self.state.sink_muted;
        cx.emit(AudioStateChanged);
        cx.notify();
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .status()
                .await;
        })
        .detach();
    }

    pub fn toggle_source_mute(&mut self, cx: &mut Context<Self>) {
        self.state.source_muted = !self.state.source_muted;
        cx.emit(AudioStateChanged);
        cx.notify();
        gpui_tokio::Tokio::spawn(cx, async move {
            let _ = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
                .status()
                .await;
        })
        .detach();
    }
}
