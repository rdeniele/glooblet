use crate::error::{AppError, AppResult};
use crate::models::MouseProfile;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

fn row_to_profile(row: &rusqlite::Row) -> rusqlite::Result<MouseProfile> {
    Ok(MouseProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        game: row.get(2)?,
        dpi: row.get(3)?,
        sensitivity: row.get(4)?,
        yaw_constant: row.get(5)?,
        pointer_speed: row.get(6)?,
        enhance_precision: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        schema_version: row.get(11)?,
    })
}

const SELECT_COLUMNS: &str = "id, name, game, dpi, sensitivity, yaw_constant, pointer_speed,
    enhance_precision, notes, created_at, updated_at, schema_version";

pub fn list(conn: &Connection) -> AppResult<Vec<MouseProfile>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_COLUMNS} FROM mouse_profiles ORDER BY updated_at DESC"
    ))?;
    let rows = stmt.query_map([], row_to_profile)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<MouseProfile> {
    conn.query_row(
        &format!("SELECT {SELECT_COLUMNS} FROM mouse_profiles WHERE id = ?1"),
        params![id],
        row_to_profile,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound(format!("profile {id}")))
}

/// Validates a profile's field ranges before it is persisted. Shared by create,
/// update, and import so every entry point enforces the same invariants.
pub fn validate(profile: &MouseProfile) -> AppResult<()> {
    if profile.name.trim().is_empty() {
        return Err(AppError::InvalidInput("profile name cannot be empty".into()));
    }
    if profile.name.len() > 80 {
        return Err(AppError::InvalidInput("profile name is too long (max 80 characters)".into()));
    }
    if let Some(dpi) = profile.dpi {
        if dpi == 0 || dpi > 32000 {
            return Err(AppError::InvalidInput(format!(
                "DPI {dpi} is out of the supported range (1-32000)"
            )));
        }
    }
    if let Some(sens) = profile.sensitivity {
        if !(sens.is_finite()) || sens <= 0.0 || sens > 1000.0 {
            return Err(AppError::InvalidInput(format!(
                "sensitivity {sens} is out of the supported range (0-1000)"
            )));
        }
    }
    if let Some(yaw) = profile.yaw_constant {
        if !(yaw.is_finite()) || yaw <= 0.0 || yaw > 10.0 {
            return Err(AppError::InvalidInput(
                "custom yaw constant must be a positive number below 10".into(),
            ));
        }
    }
    if let Some(speed) = profile.pointer_speed {
        if !(1..=20).contains(&speed) {
            return Err(AppError::InvalidInput(
                "Windows pointer speed must be between 1 and 20".into(),
            ));
        }
    }
    Ok(())
}

pub fn create(conn: &Connection, mut profile: MouseProfile) -> AppResult<MouseProfile> {
    validate(&profile)?;
    let now = Utc::now().to_rfc3339();
    profile.id = Uuid::new_v4().to_string();
    profile.created_at = now.clone();
    profile.updated_at = now;
    profile.schema_version = crate::models::SCHEMA_VERSION;

    conn.execute(
        "INSERT INTO mouse_profiles (id, name, game, dpi, sensitivity, yaw_constant,
            pointer_speed, enhance_precision, notes, created_at, updated_at, schema_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            profile.id,
            profile.name,
            profile.game,
            profile.dpi,
            profile.sensitivity,
            profile.yaw_constant,
            profile.pointer_speed,
            profile.enhance_precision,
            profile.notes,
            profile.created_at,
            profile.updated_at,
            profile.schema_version,
        ],
    )?;
    Ok(profile)
}

pub fn update(conn: &Connection, mut profile: MouseProfile) -> AppResult<MouseProfile> {
    validate(&profile)?;
    get(conn, &profile.id)?; // 404s if missing
    profile.updated_at = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE mouse_profiles SET name = ?2, game = ?3, dpi = ?4, sensitivity = ?5,
            yaw_constant = ?6, pointer_speed = ?7, enhance_precision = ?8, notes = ?9,
            updated_at = ?10
         WHERE id = ?1",
        params![
            profile.id,
            profile.name,
            profile.game,
            profile.dpi,
            profile.sensitivity,
            profile.yaw_constant,
            profile.pointer_speed,
            profile.enhance_precision,
            profile.notes,
            profile.updated_at,
        ],
    )?;
    Ok(profile)
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM mouse_profiles WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("profile {id}")));
    }
    Ok(())
}

pub fn duplicate(conn: &Connection, id: &str, new_name: String) -> AppResult<MouseProfile> {
    let mut source = get(conn, id)?;
    source.name = new_name;
    create(conn, source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn sample() -> MouseProfile {
        MouseProfile {
            id: String::new(),
            name: "CS2 — Main".into(),
            game: Some("Counter-Strike 2".into()),
            dpi: Some(800),
            sensitivity: Some(2.0),
            yaw_constant: None,
            pointer_speed: Some(10),
            enhance_precision: Some(false),
            notes: None,
            created_at: String::new(),
            updated_at: String::new(),
            schema_version: 0,
        }
    }

    #[test]
    fn create_list_update_delete_round_trip() {
        let conn = open_in_memory().unwrap();
        let created = create(&conn, sample()).unwrap();
        assert!(!created.id.is_empty());

        let listed = list(&conn).unwrap();
        assert_eq!(listed.len(), 1);

        let mut to_update = created.clone();
        to_update.dpi = Some(1600);
        let updated = update(&conn, to_update).unwrap();
        assert_eq!(updated.dpi, Some(1600));

        delete(&conn, &created.id).unwrap();
        assert!(list(&conn).unwrap().is_empty());
    }

    #[test]
    fn rejects_invalid_dpi_and_sensitivity() {
        let conn = open_in_memory().unwrap();
        let mut bad = sample();
        bad.dpi = Some(0);
        assert!(create(&conn, bad).is_err());

        let mut bad2 = sample();
        bad2.sensitivity = Some(-1.0);
        assert!(create(&conn, bad2).is_err());
    }

    #[test]
    fn duplicate_creates_independent_copy() {
        let conn = open_in_memory().unwrap();
        let created = create(&conn, sample()).unwrap();
        let dup = duplicate(&conn, &created.id, "CS2 — Copy".into()).unwrap();
        assert_ne!(dup.id, created.id);
        assert_eq!(dup.dpi, created.dpi);
        assert_eq!(list(&conn).unwrap().len(), 2);
    }

    #[test]
    fn update_missing_profile_returns_not_found() {
        let conn = open_in_memory().unwrap();
        let mut ghost = sample();
        ghost.id = "does-not-exist".into();
        assert!(matches!(update(&conn, ghost), Err(AppError::NotFound(_))));
    }
}
