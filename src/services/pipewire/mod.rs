mod audio_state;
mod device_manager;
mod stream_manager;
mod volume_control;

pub use audio_state::{AudioState, AudioDevice, AudioStream};
pub use device_manager::DeviceManager;
pub use stream_manager::StreamManager;
pub use volume_control::VolumeControl;

use glib::MainContext;
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
