pub mod profiles;
pub mod sounds;

use crate::error::AppResult;
use crate::models::AppSettings;
use rusqlite::Connection;
use std::path::Path;

pub fn open(app_data_dir: &Path) -> AppResult<Connection> {
    std::fs::create_dir_all(app_data_dir)?;
    let db_path = app_data_dir.join("glooblet.sqlite3");
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", true)?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Open an in-memory database for tests, running the same migrations as production.
#[cfg(test)]
pub fn open_in_memory() -> AppResult<Connection> {
    let conn = Connection::open_in_memory()?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if current < 1 {
        apply_migration_1(conn)?;
    }

    Ok(())
}

fn apply_migration_1(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "BEGIN;

        CREATE TABLE IF NOT EXISTS app_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            theme TEXT NOT NULL,
            launch_at_startup INTEGER NOT NULL,
            selected_profile_id TEXT,
            click_sound_enabled INTEGER NOT NULL,
            click_sound_volume REAL NOT NULL,
            desktop_sound_behavior TEXT NOT NULL,
            first_run_complete INTEGER NOT NULL,
            config_version INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS mouse_profiles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            game TEXT,
            dpi INTEGER,
            sensitivity REAL,
            yaw_constant REAL,
            pointer_speed INTEGER,
            enhance_precision INTEGER,
            notes TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            schema_version INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sound_presets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            description TEXT NOT NULL,
            file_path TEXT NOT NULL,
            is_builtin INTEGER NOT NULL,
            volume REAL NOT NULL,
            duration_ms INTEGER,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS click_sound_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            left_click_preset_id TEXT,
            right_click_preset_id TEXT,
            middle_click_preset_id TEXT
        );

        INSERT INTO schema_migrations (version) VALUES (1);
        COMMIT;",
    )?;
    Ok(())
}

pub fn load_settings(conn: &Connection) -> AppResult<AppSettings> {
    let result = conn.query_row(
        "SELECT theme, launch_at_startup, selected_profile_id, click_sound_enabled,
                click_sound_volume, desktop_sound_behavior, first_run_complete, config_version
         FROM app_settings WHERE id = 1",
        [],
        |row| {
            Ok(AppSettings {
                theme: row.get(0)?,
                launch_at_startup: row.get(1)?,
                selected_profile_id: row.get(2)?,
                click_sound_enabled: row.get(3)?,
                click_sound_volume: row.get(4)?,
                desktop_sound_behavior: row.get(5)?,
                first_run_complete: row.get(6)?,
                config_version: row.get(7)?,
            })
        },
    );

    match result {
        Ok(settings) => Ok(settings),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let defaults = AppSettings::default();
            save_settings(conn, &defaults)?;
            Ok(defaults)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn save_settings(conn: &Connection, settings: &AppSettings) -> AppResult<()> {
    conn.execute(
        "INSERT INTO app_settings (id, theme, launch_at_startup, selected_profile_id,
            click_sound_enabled, click_sound_volume, desktop_sound_behavior,
            first_run_complete, config_version)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
            theme = excluded.theme,
            launch_at_startup = excluded.launch_at_startup,
            selected_profile_id = excluded.selected_profile_id,
            click_sound_enabled = excluded.click_sound_enabled,
            click_sound_volume = excluded.click_sound_volume,
            desktop_sound_behavior = excluded.desktop_sound_behavior,
            first_run_complete = excluded.first_run_complete,
            config_version = excluded.config_version",
        rusqlite::params![
            settings.theme,
            settings.launch_at_startup,
            settings.selected_profile_id,
            settings.click_sound_enabled,
            settings.click_sound_volume,
            settings.desktop_sound_behavior,
            settings.first_run_complete,
            settings.config_version,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_idempotent() {
        let conn = open_in_memory().unwrap();
        // Running again must not fail or duplicate tables.
        run_migrations(&conn).unwrap();
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, crate::models::SCHEMA_VERSION);
    }

    #[test]
    fn settings_round_trip_with_defaults_on_first_read() {
        let conn = open_in_memory().unwrap();
        let loaded = load_settings(&conn).unwrap();
        assert_eq!(loaded.theme, "dark");
        assert!(!loaded.first_run_complete);

        let mut updated = loaded;
        updated.first_run_complete = true;
        updated.click_sound_volume = 0.42;
        save_settings(&conn, &updated).unwrap();

        let reloaded = load_settings(&conn).unwrap();
        assert!(reloaded.first_run_complete);
        assert!((reloaded.click_sound_volume - 0.42).abs() < f32::EPSILON);
    }
}
