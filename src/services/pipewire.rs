use std::process::Command;
use std::sync::mpsc;
use glib::{MainContext, ControlFlow, Priority};

#[derive(Debug, Clone)]
pub struct AudioState {
    pub volume: u8,
    pub muted: bool,
    pub mic_volume: u8,
    pub mic_muted: bool,
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
                match String::from_utf8(output.stdout) {
                    Ok(output_str) => {
                        if let Some(volume_str) = output_str.split_whitespace().nth(1) {
                            if let Ok(volume) = volume_str.parse::<f32>() {
                                return (volume * 100.0) as u8;
                            }
                        }
                    }
                    Err(_) => {}
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

        let (tx_glib, rx_glib) = MainContext::channel(Priority::DEFAULT);

        std::thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                if tx_glib.send(state).is_err() {
                    break;
                }
            }
        });

        rx_glib.attach(None, move |state| {
            callback(state);
            ControlFlow::Continue
        });
    }
}
