use crate::services::database::get_database;
use crate::widgets::r#macro::types::*;
use anyhow::Result;
use uuid::Uuid;

pub fn load_macros() -> Result<Vec<Macro>> {
    let db = get_database();
    let conn = db.conn();
    let conn = conn.lock();

    let mut stmt = conn
        .prepare("SELECT id, name, app_class, created_at FROM macros ORDER BY created_at DESC")?;

    let macros = stmt
        .query_map([], |row| {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        e.to_string(),
                    )),
                )
            })?;
            let name: String = row.get(1)?;
            let app_class: Option<String> = row.get(2)?;
            let created_at: u64 = row.get(3)?;

            let actions = load_actions(&conn, &id_str)?;

            Ok(Macro {
                id,
                name,
                app_class,
                actions,
                created_at,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(macros)
}

pub fn load_actions(
    conn: &rusqlite::Connection,
    macro_id: &str,
) -> Result<Vec<MacroAction>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT timestamp_ms, action_type, action_data, click_zone_x, click_zone_y, 
                click_zone_width, click_zone_height 
         FROM macro_actions 
         WHERE macro_id = ? 
         ORDER BY action_index",
    )?;

    let actions = stmt
        .query_map([macro_id], |row| {
            let timestamp_ms: u64 = row.get(0)?;
            let action_type_str: String = row.get(1)?;
            let action_data: Option<String> = row.get(2)?;

            let action_type =
                parse_action_type(&action_type_str, action_data.as_deref()).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e.to_string(),
                        )),
                    )
                })?;

            let click_zone = if let (Some(x), Some(y), Some(w), Some(h)) = (
                row.get::<_, Option<i32>>(3)?,
                row.get::<_, Option<i32>>(4)?,
                row.get::<_, Option<u32>>(5)?,
                row.get::<_, Option<u32>>(6)?,
            ) {
                Some(ClickZone {
                    x,
                    y,
                    width: w,
                    height: h,
                })
            } else {
                None
            };

            Ok(MacroAction {
                timestamp_ms,
                action_type,
                click_zone,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(actions)
}

pub fn parse_action_type(type_str: &str, data: Option<&str>) -> Result<ActionType> {
    match type_str {
        "KeyPress" => {
            let code = data
                .ok_or_else(|| anyhow::anyhow!("Missing data for KeyPress"))?
                .parse::<u32>()?;
            Ok(ActionType::KeyPress(code))
        }
        "KeyRelease" => {
            let code = data
                .ok_or_else(|| anyhow::anyhow!("Missing data for KeyRelease"))?
                .parse::<u32>()?;
            Ok(ActionType::KeyRelease(code))
        }
        "MouseClick" => {
            let btn_str = data.ok_or_else(|| anyhow::anyhow!("Missing data for MouseClick"))?;
            let btn = match btn_str {
                "Left" => MacroMouseButton::Left,
                "Right" => MacroMouseButton::Right,
                "Middle" => MacroMouseButton::Middle,
                _ => MacroMouseButton::Left,
            };
            Ok(ActionType::MouseClick(btn))
        }
        "Delay" => {
            let ms = data
                .ok_or_else(|| anyhow::anyhow!("Missing data for Delay"))?
                .parse::<u64>()?;
            Ok(ActionType::Delay(ms))
        }
        _ => Err(anyhow::anyhow!("Unknown action type: {}", type_str)),
    }
}

pub fn save_macros_sync(macros: Vec<Macro>) -> Result<()> {
    let db = get_database();
    let conn = db.conn();
    let conn = conn.lock();

    conn.execute("DELETE FROM macro_actions", [])?;
    conn.execute("DELETE FROM macros", [])?;

    for macro_rec in macros {
        conn.execute(
            "INSERT INTO macros (id, name, app_class, created_at) VALUES (?, ?, ?, ?)",
            rusqlite::params![
                macro_rec.id.to_string(),
                macro_rec.name,
                macro_rec.app_class,
                macro_rec.created_at,
            ],
        )?;

        for (idx, action) in macro_rec.actions.iter().enumerate() {
            let (action_type_str, action_data) = serialize_action_type(&action.action_type);

            conn.execute(
                "INSERT INTO macro_actions 
                 (macro_id, action_index, timestamp_ms, action_type, action_data, 
                  click_zone_x, click_zone_y, click_zone_width, click_zone_height)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    macro_rec.id.to_string(),
                    idx as i64,
                    action.timestamp_ms as i64,
                    action_type_str,
                    action_data,
                    action.click_zone.as_ref().map(|z| z.x),
                    action.click_zone.as_ref().map(|z| z.y),
                    action.click_zone.as_ref().map(|z| z.width as i64),
                    action.click_zone.as_ref().map(|z| z.height as i64),
                ],
            )?;
        }
    }

    Ok(())
}

pub fn serialize_action_type(action_type: &ActionType) -> (&'static str, String) {
    match action_type {
        ActionType::KeyPress(code) => ("KeyPress", code.to_string()),
        ActionType::KeyRelease(code) => ("KeyRelease", code.to_string()),
        ActionType::MouseClick(btn) => {
            let btn_str = match btn {
                MacroMouseButton::Left => "Left",
                MacroMouseButton::Right => "Right",
                MacroMouseButton::Middle => "Middle",
            };
            ("MouseClick", btn_str.to_string())
        }
        ActionType::Delay(ms) => ("Delay", ms.to_string()),
    }
}
