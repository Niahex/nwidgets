use crate::widgets::r#macro::types::*;
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

pub async fn replay_macro(
    macro_rec: &Macro,
    playing: Arc<RwLock<Option<Uuid>>>,
    playback_speed: Arc<RwLock<f32>>,
) -> Result<()> {
    let mut wayland_input = super::wayland_input::WaylandInput::new()?;
    let mut last_timestamp = 0u64;
    let mut current_mouse_x = 0i32;
    let mut current_mouse_y = 0i32;

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
                    let (target_x, target_y) = randomize_click_position(zone);
                    if let Err(e) = wayland_input.move_pointer_smooth(
                        target_x,
                        target_y,
                        current_mouse_x,
                        current_mouse_y,
                    ) {
                        log::warn!("Failed to move pointer smoothly: {}", e);
                    }
                    current_mouse_x = target_x;
                    current_mouse_y = target_y;
                    
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
                
                if let Err(e) = wayland_input.click_button(*btn) {
                    log::warn!("Failed to click button: {}", e);
                }
            }
            ActionType::KeyPress(code) | ActionType::KeyRelease(code) => {
                let pressed = matches!(action.action_type, ActionType::KeyPress(_));
                if let Err(e) = wayland_input.send_key(*code, pressed) {
                    log::warn!("Failed to send key: {}", e);
                }
            }
            ActionType::Delay(_) => {
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
