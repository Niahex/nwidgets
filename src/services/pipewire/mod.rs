mod audio_state;
mod device_manager;
mod stream_manager;
mod volume_control;

pub use audio_state::{AudioDevice, AudioState, AudioStream};
pub use device_manager::DeviceManager;
pub use stream_manager::StreamManager;
pub use volume_control::VolumeControl;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;

pub struct PipeWireService;

impl PipeWireService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_volume(&self) -> u8 {
        VolumeControl::get_volume()
    }

    pub fn is_muted(&self) -> bool {
        VolumeControl::is_muted()
    }

    pub fn get_mic_volume(&self) -> u8 {
        VolumeControl::get_mic_volume()
    }

    pub fn is_mic_muted(&self) -> bool {
        VolumeControl::is_mic_muted()
    }

    pub fn set_volume(volume: u8) {
        VolumeControl::set_volume(volume);
    }

    pub fn set_mic_volume(volume: u8) {
        VolumeControl::set_mic_volume(volume);
    }

    pub fn toggle_mute() {
        VolumeControl::toggle_mute();
    }

    pub fn toggle_mic_mute() {
        VolumeControl::toggle_mic_mute();
    }

    pub fn get_audio_state() -> AudioState {
        VolumeControl::get_audio_state()
    }

    pub fn list_sinks() -> Vec<AudioDevice> {
        DeviceManager::list_sinks()
    }

    pub fn list_sources() -> Vec<AudioDevice> {
        DeviceManager::list_sources()
    }

    pub fn list_sink_inputs() -> Vec<AudioStream> {
        StreamManager::list_sink_inputs()
    }

    pub fn list_source_outputs() -> Vec<AudioStream> {
        StreamManager::list_source_outputs()
    }

    pub fn set_stream_volume(stream_id: u32, volume: u8) {
        StreamManager::set_stream_volume(stream_id, volume);
    }

    pub fn toggle_stream_mute(stream_id: u32) {
        StreamManager::toggle_stream_mute(stream_id);
    }

    pub fn set_default_sink(sink_id: u32) {
        DeviceManager::set_default_sink(sink_id);
    }

    pub fn set_default_source(source_id: u32) {
        DeviceManager::set_default_source(source_id);
    }

    pub fn subscribe_audio<F>(callback: F)
    where
        F: Fn(AudioState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let mut last_state = Self::get_audio_state();
            if tx.send(last_state.clone()).is_err() {
                return;
            }

            // Start pw-mon process to monitor changes
            let mut child = match Command::new("pw-mon").stdout(Stdio::piped()).spawn() {
                Ok(child) => child,
                Err(e) => {
                    eprintln!("Failed to start pw-mon: {e}. Falling back to polling.");
                    // Fallback polling loop
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                        let new_state = Self::get_audio_state();
                        if new_state != last_state {
                            if tx.send(new_state.clone()).is_err() {
                                break;
                            }
                            last_state = new_state;
                        }
                    }
                    return;
                }
            };

            let stdout = child
                .stdout
                .take()
                .expect("Failed to capture pw-mon stdout");
            let reader = BufReader::new(stdout);

            // Channel to signal that an event occurred
            let (event_tx, event_rx) = mpsc::channel();

            // Spawn a thread to read pw-mon output
            std::thread::spawn(move || {
                for line in reader.lines() {
                    if let Ok(l) = line {
                        // "changed:" indicates a state change in the PipeWire graph
                        if l.trim().starts_with("changed:") && event_tx.send(()).is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            });

            loop {
                // Wait for an event (blocking)
                if event_rx.recv().is_err() {
                    break;
                }

                // Debounce: Wait 50ms to coalesce rapid events (like volume sliding)
                std::thread::sleep(std::time::Duration::from_millis(50));

                // Drain any other events that came in during the sleep
                while event_rx.try_recv().is_ok() {}

                let new_state = Self::get_audio_state();
                if new_state != last_state {
                    if tx.send(new_state.clone()).is_err() {
                        break;
                    }
                    last_state = new_state;
                }
            }

            // Cleanup
            let _ = child.kill();
        });

        crate::utils::subscription::ServiceSubscription::subscribe(rx, callback);
    }
}
