use crate::widgets::r#macro::types::*;
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::process::Command;
use uuid::Uuid;

pub async fn replay_macro(
    macro_rec: &Macro,
    playing: Arc<RwLock<Option<Uuid>>>,
    playback_speed: Arc<RwLock<f32>>,
) -> Result<()> {
    let mut last_timestamp = 0u64;

    for action in &macro_rec.actions {
        if playing.read().is_none() {
            break;
        }

        let delay_ms = action.timestamp_ms.saturating_sub(last_timestamp);
        let speed = *playback_speed.read();
        let adjusted_delay = (delay_ms as f32 / speed) as u64;

        if adjusted_delay > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(adjusted_delay)).await;
        }

        match &action.action_type {
            ActionType::MouseClick(btn) => {
                if let Some(zone) = &action.click_zone {
                    let (x, y) = randomize_click_position(zone);
                    if let Err(e) = Command::new("ydotool")
                        .args(["mousemove", "--absolute", &x.to_string(), &y.to_string()])
                        .output()
                        .await
                    {
                        log::warn!("Failed to execute ydotool mousemove: {}", e);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
                
                let btn_code = match btn {
                    MacroMouseButton::Left => "0xC0",
                    MacroMouseButton::Right => "0xC1",
                    MacroMouseButton::Middle => "0xC2",
                };
                if let Err(e) = Command::new("ydotool")
                    .args(["click", btn_code])
                    .output()
                    .await
                {
                    log::warn!("Failed to execute ydotool click: {}", e);
                }
            }
            ActionType::KeyPress(code) | ActionType::KeyRelease(code) => {
                if let Err(e) = Command::new("ydotool")
                    .args(["key", &code.to_string()])
                    .output()
                    .await
                {
                    log::warn!("Failed to execute ydotool key: {}", e);
                }
            }
            ActionType::Delay(_) => {
                // Delay is already handled by the timestamp difference calculation above
            }
        }

        last_timestamp = action.timestamp_ms;
    }

    Ok(())
}

pub fn randomize_click_position(zone: &ClickZone) -> (i32, i32) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let x = zone.x + rng.gen_range(0..zone.width as i32);
    let y = zone.y + rng.gen_range(0..zone.height as i32);
    (x, y)
}
