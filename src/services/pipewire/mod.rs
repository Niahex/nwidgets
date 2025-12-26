mod audio_state;
mod device_manager;
mod pw_dump;
mod debug;
mod stream_manager;
mod volume_control;

pub use audio_state::{AudioDevice, AudioState, AudioStream};
pub use device_manager::DeviceManager;
pub use stream_manager::StreamManager;
pub use volume_control::VolumeControl;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use crate::services::pipewire::pw_dump::PipeWireObject;
use serde_json;

pub struct PipeWireService;

impl PipeWireService {
    pub fn set_volume(volume: u8) {
        crate::utils::runtime::get().spawn(async move {
            VolumeControl::set_volume(volume);
        });
    }

    pub fn set_mic_volume(volume: u8) {
        crate::utils::runtime::get().spawn(async move {
            VolumeControl::set_mic_volume(volume);
        });
    }

    pub fn get_audio_state() -> AudioState {
        // Fallback for synchronous callers (should be avoided in UI loops)
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
        crate::utils::runtime::get().spawn(async move {
            StreamManager::set_stream_volume(stream_id, volume);
        });
    }

    pub fn toggle_stream_mute(stream_id: u32) {
        crate::utils::runtime::get().spawn(async move {
            StreamManager::toggle_stream_mute(stream_id);
        });
    }

    pub fn set_default_sink(sink_id: u32) {
        crate::utils::runtime::get().spawn(async move {
            DeviceManager::set_default_sink(sink_id);
        });
    }

    pub fn set_default_source(source_id: u32) {
        crate::utils::runtime::get().spawn(async move {
            DeviceManager::set_default_source(source_id);
        });
    }

    fn parse_dump() -> Option<AudioState> {
        let output = Command::new("pw-dump")
            .output()
            .ok()?;
        
        let objects: Vec<PipeWireObject> = serde_json::from_slice(&output.stdout).ok()?;
        
        // Find default devices from metadata
        let mut default_sink_name = String::new();
        let mut default_source_name = String::new();
        
        for obj in &objects {
            if obj.type_ == "PipeWire:Interface:Metadata" {
                if let Some(metadata) = &obj.metadata {
                    for entry in metadata {
                        if entry.key == "default.audio.sink" {
                             if let Some(val) = entry.value.get("name").and_then(|v| v.as_str()) {
                                 default_sink_name = val.to_string();
                             }
                        }
                        if entry.key == "default.audio.source" {
                             if let Some(val) = entry.value.get("name").and_then(|v| v.as_str()) {
                                 default_source_name = val.to_string();
                             }
                        }
                    }
                }
            }
        }

        let mut sinks = Vec::new();
        let mut sources = Vec::new();
        let mut sink_inputs = Vec::new();
        let mut source_outputs = Vec::new();

        let mut master_volume = 0;
        let mut master_muted = false;
        let mut mic_volume = 0;
        let mut mic_muted = false;

        for obj in &objects {
             // Only interested in Nodes
            if obj.type_ != "PipeWire:Interface:Node" {
                continue;
            }

            let media_class = match obj.get_media_class() {
                Some(mc) => mc,
                None => continue,
            };
            
            let id = obj.id;
            let (vol_pct, muted) = obj.get_volume_info();
            
            match media_class.as_str() {
                "Audio/Sink" => {
                    let name = obj.get_node_name().unwrap_or_default();
                    let desc = obj.get_node_desc().unwrap_or(name.clone());
                    let is_default = name == default_sink_name;
                    
                    if is_default {
                        master_volume = vol_pct;
                        master_muted = muted;
                    }
                    
                    sinks.push(AudioDevice {
                        id,
                        description: desc,
                        is_default,
                    });
                },
                "Audio/Source" => {
                    let name = obj.get_node_name().unwrap_or_default();
                    let desc = obj.get_node_desc().unwrap_or(name.clone());
                    let is_default = name == default_source_name;

                    if is_default {
                        mic_volume = vol_pct;
                        mic_muted = muted;
                    }

                    sources.push(AudioDevice {
                        id,
                        description: desc,
                        is_default,
                    });
                },
                mc if mc.contains("Stream") => { 
                    let app_name = obj.get_app_name().unwrap_or_else(|| "Unknown App".to_string());
                    let app_icon = obj.get_app_icon_name();
                    
                    let stream = AudioStream {
                        id,
                        app_name,
                        volume: vol_pct,
                        muted,
                        window_title: None,
                        app_icon,
                    };

                    // PipeWire terminology:
                    // Stream/Output/Audio = Application PRODUCING audio (goes to Sink) -> Sink Input
                    // Stream/Input/Audio  = Application CONSUMING audio (comes from Source) -> Source Output
                    
                    if mc.contains("Output") {
                        sink_inputs.push(stream);
                    } else if mc.contains("Input") {
                        // Filter out internal streams like bluetooth capture
                        if !mc.contains("Internal") {
                             source_outputs.push(stream);
                        }
                    }
                },
                _ => {}
            }
        }
        
        Some(AudioState {
            volume: master_volume,
            muted: master_muted,
            mic_volume,
            mic_muted,
            sinks,
            sources,
            sink_inputs,
            source_outputs,
        })
    }

    pub fn subscribe_audio<F>(callback: F)
    where
        F: Fn(AudioState) + 'static,
    {
        let (tx, rx) = mpsc::channel();

        // Print debug info to console at startup
        debug::debug_dump();

        std::thread::spawn(move || {
            // Initial state
            if let Some(state) = Self::parse_dump() {
                let _ = tx.send(state);
            }

            // Start pw-mon process to monitor changes
            let mut child = match Command::new("pw-mon").stdout(Stdio::piped()).spawn() {
                Ok(child) => child,
                Err(e) => {
                    eprintln!("Failed to start pw-mon: {e}. Falling back to polling.");
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(2000));
                        if let Some(new_state) = Self::parse_dump() {
                            if tx.send(new_state).is_err() {
                                break;
                            }
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

            let (event_tx, event_rx) = mpsc::channel();

            std::thread::spawn(move || {
                for line in reader.lines() {
                    if let Ok(l) = line {
                        if l.trim().starts_with("changed:") && event_tx.send(()).is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            });

            loop {
                if event_rx.recv().is_err() {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(50));
                while event_rx.try_recv().is_ok() {}

                if let Some(new_state) = Self::parse_dump() {
                     if tx.send(new_state).is_err() {
                        break;
                    }
                }
            }

            let _ = child.kill();
        });

        crate::utils::subscription::ServiceSubscription::subscribe(rx, callback);
    }
}