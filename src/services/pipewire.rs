use std::process::Command;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct PipeWireService {
    volume: u8,
    muted: bool,
}

impl PipeWireService {
    pub fn new() -> Self {
        println!("[PIPEWIRE_SERVICE] 🎵 Creating PipeWireService");
        Self {
            volume: 0,
            muted: false,
        }
    }

    /// Start monitoring volume changes and send updates through the channel
    pub fn start_monitoring() -> mpsc::UnboundedReceiver<(u8, bool)> {
        println!("[PIPEWIRE_SERVICE] 🔊 Starting volume monitoring");
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let service = PipeWireService::new();
            let mut last_volume = service.get_volume();
            let mut last_muted = service.is_muted();
            println!("[PIPEWIRE_SERVICE] 📊 Initial state - volume: {}, muted: {}", last_volume, last_muted);

            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let current_volume = service.get_volume();
                let current_muted = service.is_muted();

                if current_volume != last_volume || current_muted != last_muted {
                    println!("[PIPEWIRE_SERVICE] 🔔 Volume changed: {}% (muted: {}) -> {}% (muted: {})",
                        last_volume, last_muted, current_volume, current_muted);

                    if tx.send((current_volume, current_muted)).is_err() {
                        println!("[PIPEWIRE_SERVICE] ⚠️  Receiver dropped, stopping monitoring");
                        break;
                    }

                    last_volume = current_volume;
                    last_muted = current_muted;
                }
            }
        });

        rx
    }

    pub fn get_volume(&self) -> u8 {
        match Command::new("wpctl")
            .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            Ok(output) => {
                match String::from_utf8(output.stdout) {
                    Ok(output_str) => {
                        println!("[PIPEWIRE_SERVICE] 🔊 wpctl output: '{}'", output_str.trim());
                        if let Some(volume_str) = output_str.split_whitespace().nth(1) {
                            match volume_str.parse::<f32>() {
                                Ok(volume) => {
                                    let volume_percent = (volume * 100.0) as u8;
                                    println!("[PIPEWIRE_SERVICE] 🔊 Parsed volume: {}% (from {})", volume_percent, volume);
                                    return volume_percent;
                                }
                                Err(e) => {
                                    println!("[PIPEWIRE_SERVICE] ❌ Failed to parse volume: {}", e);
                                }
                            }
                        } else {
                            println!("[PIPEWIRE_SERVICE] ⚠️  Could not find volume value in output");
                        }
                    }
                    Err(e) => {
                        println!("[PIPEWIRE_SERVICE] ❌ Failed to parse output as UTF-8: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("[PIPEWIRE_SERVICE] ❌ wpctl command failed: {}", e);
            }
        }
        println!("[PIPEWIRE_SERVICE] ⚠️  Returning default volume: 0");
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
}
