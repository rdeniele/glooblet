use crate::error::{AppError, AppResult};
use crate::models::{ClickSoundConfig, SoundPreset};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

fn row_to_preset(row: &rusqlite::Row) -> rusqlite::Result<SoundPreset> {
    Ok(SoundPreset {
        id: row.get(0)?,
        name: row.get(1)?,
        category: row.get(2)?,
        description: row.get(3)?,
        file_path: row.get(4)?,
        is_builtin: row.get(5)?,
        volume: row.get(6)?,
        duration_ms: row.get(7)?,
        created_at: row.get(8)?,
    })
}

const COLS: &str = "id, name, category, description, file_path, is_builtin, volume, duration_ms, created_at";

pub fn list(conn: &Connection) -> AppResult<Vec<SoundPreset>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLS} FROM sound_presets ORDER BY is_builtin DESC, created_at ASC"))?;
    let rows = stmt.query_map([], row_to_preset)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<SoundPreset> {
    conn.query_row(&format!("SELECT {COLS} FROM sound_presets WHERE id = ?1"), params![id], row_to_preset)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("sound preset {id}")))
}

pub fn insert(conn: &Connection, mut preset: SoundPreset) -> AppResult<SoundPreset> {
    if preset.name.trim().is_empty() {
        return Err(AppError::InvalidInput("sound name cannot be empty".into()));
    }
    if preset.id.is_empty() {
        preset.id = Uuid::new_v4().to_string();
    }
    if preset.created_at.is_empty() {
        preset.created_at = Utc::now().to_rfc3339();
    }
    conn.execute(
        "INSERT INTO sound_presets (id, name, category, description, file_path, is_builtin, volume, duration_ms, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            preset.id, preset.name, preset.category, preset.description, preset.file_path,
            preset.is_builtin, preset.volume, preset.duration_ms, preset.created_at,
        ],
    )?;
    Ok(preset)
}

pub fn delete_custom(conn: &Connection, id: &str) -> AppResult<SoundPreset> {
    let preset = get(conn, id)?;
    if preset.is_builtin {
        return Err(AppError::InvalidInput("built-in sounds cannot be removed".into()));
    }
    conn.execute("DELETE FROM sound_presets WHERE id = ?1", params![id])?;
    // Clear any click-sound slot referencing this preset so the config never
    // points at a dangling id.
    conn.execute(
        "UPDATE click_sound_config SET left_click_preset_id = NULL WHERE left_click_preset_id = ?1",
        params![id],
    )?;
    conn.execute(
        "UPDATE click_sound_config SET right_click_preset_id = NULL WHERE right_click_preset_id = ?1",
        params![id],
    )?;
    conn.execute(
        "UPDATE click_sound_config SET middle_click_preset_id = NULL WHERE middle_click_preset_id = ?1",
        params![id],
    )?;
    Ok(preset)
}

pub fn get_click_config(conn: &Connection) -> AppResult<ClickSoundConfig> {
    let result = conn.query_row(
        "SELECT left_click_preset_id, right_click_preset_id, middle_click_preset_id FROM click_sound_config WHERE id = 1",
        [],
        |row| {
            Ok(ClickSoundConfig {
                left_click_preset_id: row.get(0)?,
                right_click_preset_id: row.get(1)?,
                middle_click_preset_id: row.get(2)?,
            })
        },
    );
    match result {
        Ok(cfg) => Ok(cfg),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(ClickSoundConfig::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_click_config(conn: &Connection, cfg: &ClickSoundConfig) -> AppResult<()> {
    conn.execute(
        "INSERT INTO click_sound_config (id, left_click_preset_id, right_click_preset_id, middle_click_preset_id)
         VALUES (1, ?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET
            left_click_preset_id = excluded.left_click_preset_id,
            right_click_preset_id = excluded.right_click_preset_id,
            middle_click_preset_id = excluded.middle_click_preset_id",
        params![cfg.left_click_preset_id, cfg.right_click_preset_id, cfg.middle_click_preset_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn sample(name: &str, builtin: bool) -> SoundPreset {
        SoundPreset {
            id: String::new(),
            name: name.into(),
            category: "soft".into(),
            description: "test sound".into(),
            file_path: "sounds/test.wav".into(),
            is_builtin: builtin,
            volume: 0.8,
            duration_ms: Some(120),
            created_at: String::new(),
        }
    }

    #[test]
    fn insert_and_list() {
        let conn = open_in_memory().unwrap();
        insert(&conn, sample("Soft Pop", true)).unwrap();
        insert(&conn, sample("My Click", false)).unwrap();
        assert_eq!(list(&conn).unwrap().len(), 2);
    }

    #[test]
    fn cannot_delete_builtin() {
        let conn = open_in_memory().unwrap();
        let p = insert(&conn, sample("Soft Pop", true)).unwrap();
        assert!(delete_custom(&conn, &p.id).is_err());
    }

    #[test]
    fn deleting_custom_sound_clears_click_config_reference() {
        let conn = open_in_memory().unwrap();
        let p = insert(&conn, sample("My Click", false)).unwrap();
        save_click_config(
            &conn,
            &ClickSoundConfig {
                left_click_preset_id: Some(p.id.clone()),
                right_click_preset_id: None,
                middle_click_preset_id: None,
            },
        )
        .unwrap();
        delete_custom(&conn, &p.id).unwrap();
        let cfg = get_click_config(&conn).unwrap();
        assert_eq!(cfg.left_click_preset_id, None);
    }
}
