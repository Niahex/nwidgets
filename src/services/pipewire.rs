use crate::services::osd::{OsdEvent, OsdEventService};
use glib::MainContext;
use std::process::Command;
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AudioState {
    pub volume: u8,
    pub muted: bool,
    pub mic_volume: u8,
    pub mic_muted: bool,
}

impl AudioState {
    /// Retourne le nom de l'icône pour le sink (sortie audio) en fonction du volume et de l'état muted
    pub fn get_sink_icon_name(&self) -> &'static str {
        if self.muted {
            "sink-muted"
        } else if self.volume == 0 {
            "sink-zero"
        } else if self.volume < 33 {
            "sink-low"
        } else if self.volume < 66 {
            "sink-medium"
        } else {
            "sink-high"
        }
    }

    /// Retourne le nom de l'icône pour la source (entrée audio/micro) en fonction du volume et de l'état muted
    pub fn get_source_icon_name(&self) -> &'static str {
        if self.mic_muted {
            "source-muted"
        } else if self.mic_volume == 0 {
            "source-zero"
        } else if self.mic_volume < 33 {
            "source-low"
        } else if self.mic_volume < 66 {
            "source-medium"
        } else {
            "source-high"
        }
    }
}

pub struct PipeWireService;

impl PipeWireService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_volume(&self) -> u8 {
        match Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            Ok(output) => {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(volume_str) = output_str.split_whitespace().nth(1) {
                        if let Ok(volume) = volume_str.parse::<f32>() {
                            return (volume * 100.0) as u8;
                        }
                    }
                }
            }
            Err(_) => {}
        }
        0
    }

    pub fn is_muted(&self) -> bool {
        if let Ok(output) = Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                return output_str.contains("[MUTED]");
            }
        }
        false
    }

    pub fn get_mic_volume(&self) -> u8 {
        match Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
            .output()
        {
            Ok(output) => {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(volume_str) = output_str.split_whitespace().nth(1) {
                        if let Ok(volume) = volume_str.parse::<f32>() {
                            return (volume * 100.0) as u8;
                        }
                    }
                }
            }
            Err(_) => {}
        }
        0
    }

    pub fn is_mic_muted(&self) -> bool {
        if let Ok(output) = Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                return output_str.contains("[MUTED]");
            }
        }
        false
    }

    pub fn set_volume(volume: u8) {
        let volume_val = volume.min(100);
        let volume_str = format!("{}%", volume_val);
        let _ = Command::new("wpctl")
            .args(&["set-volume", "@DEFAULT_AUDIO_SINK@", &volume_str])
            .output();

        let state = Self::get_audio_state();
        let icon_name = state.get_sink_icon_name();
        OsdEventService::send_event(OsdEvent::Volume(icon_name.to_string(), volume_val, state.muted));
    }

    pub fn set_mic_volume(volume: u8) {
        let volume_val = volume.min(100);
        let volume_str = format!("{}%", volume_val);
        let _ = Command::new("wpctl")
            .args(&["set-volume", "@DEFAULT_AUDIO_SOURCE@", &volume_str])
            .output();

        let state = Self::get_audio_state();
        let icon_name = state.get_source_icon_name();
        OsdEventService::send_event(OsdEvent::Volume(icon_name.to_string(), volume_val, state.mic_muted));
    }

    fn get_audio_state() -> AudioState {
        let service = Self::new();
        AudioState {
            volume: service.get_volume(),
            muted: service.is_muted(),
            mic_volume: service.get_mic_volume(),
            mic_muted: service.is_mic_muted(),
        }
    }

    pub fn subscribe_audio<F>(callback: F)
    where
        F: Fn(AudioState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let mut last_state = Self::get_audio_state();
            let _ = tx.send(last_state.clone());

            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let new_state = Self::get_audio_state();

                if new_state.volume != last_state.volume
                    || new_state.muted != last_state.muted
                    || new_state.mic_volume != last_state.mic_volume
                    || new_state.mic_muted != last_state.mic_muted
                {
                    if tx.send(new_state.clone()).is_err() {
                        break;
                    }
                    last_state = new_state;
                }
            }
        });

        let (async_tx, async_rx) = async_channel::unbounded();

        std::thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                if async_tx.send_blocking(state).is_err() {
                    break;
                }
            }
        });

        MainContext::default().spawn_local(async move {
            while let Ok(state) = async_rx.recv().await {
                callback(state);
            }
        });
    }
}