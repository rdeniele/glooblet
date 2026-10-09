use crate::calc::{self, Cm360Result};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{GameFormula, MouseProfile};
use crate::state::AppState;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn list_profiles(state: State<AppState>) -> AppResult<Vec<MouseProfile>> {
    let conn = state.db.lock().unwrap();
    db::profiles::list(&conn)
}

#[tauri::command]
pub fn get_profile(state: State<AppState>, id: String) -> AppResult<MouseProfile> {
    let conn = state.db.lock().unwrap();
    db::profiles::get(&conn, &id)
}

#[tauri::command]
pub fn create_profile(state: State<AppState>, profile: MouseProfile) -> AppResult<MouseProfile> {
    let conn = state.db.lock().unwrap();
    db::profiles::create(&conn, profile)
}

#[tauri::command]
pub fn update_profile(state: State<AppState>, profile: MouseProfile) -> AppResult<MouseProfile> {
    let conn = state.db.lock().unwrap();
    db::profiles::update(&conn, profile)
}

#[tauri::command]
pub fn delete_profile(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    db::profiles::delete(&conn, &id)
}

#[tauri::command]
pub fn duplicate_profile(
    state: State<AppState>,
    id: String,
    new_name: String,
) -> AppResult<MouseProfile> {
    let conn = state.db.lock().unwrap();
    db::profiles::duplicate(&conn, &id, new_name)
}

/// Exports a single profile as a JSON string the user saves themselves
/// (the frontend triggers the save dialog; the backend never writes outside
/// app-controlled directories on its own).
#[tauri::command]
pub fn export_profile(state: State<AppState>, id: String) -> AppResult<String> {
    let conn = state.db.lock().unwrap();
    let profile = db::profiles::get(&conn, &id)?;
    serde_json::to_string_pretty(&profile)
        .map_err(|e| AppError::InvalidInput(format!("failed to serialize profile: {e}")))
}

/// Imports a profile from a JSON string, re-validating every field exactly
/// as a manually created profile would be (never trusts the file's values).
#[tauri::command]
pub fn import_profile(state: State<AppState>, json: String) -> AppResult<MouseProfile> {
    let mut parsed: MouseProfile = serde_json::from_str(&json)
        .map_err(|e| AppError::InvalidInput(format!("not a valid Glooblet profile file: {e}")))?;
    // Treat the import as a brand-new profile: ignore any id/timestamps in the file.
    parsed.id = String::new();
    parsed.created_at = String::new();
    parsed.updated_at = String::new();
    let conn = state.db.lock().unwrap();
    db::profiles::create(&conn, parsed)
}

/// Opens a native save dialog and writes the profile's JSON to exactly the
/// path the user picked. Returns `None` if the user cancelled.
#[tauri::command]
pub fn export_profile_to_file(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: String,
    suggested_name: String,
) -> AppResult<Option<String>> {
    let json = export_profile(state, id)?;

    let picked = app_handle
        .dialog()
        .file()
        .add_filter("Glooblet Profile", &["json"])
        .set_file_name(&format!("{suggested_name}.json"))
        .blocking_save_file();

    let Some(file_path) = picked else {
        return Ok(None);
    };
    let dest = file_path
        .into_path()
        .map_err(|e| AppError::InvalidInput(format!("invalid file path: {e}")))?;
    std::fs::write(&dest, json)?;
    Ok(Some(dest.to_string_lossy().to_string()))
}

/// Opens a native "open file" dialog scoped to `.json`, reads whatever the
/// user picked, and imports it through the same validation path as
/// `import_profile`. Returns `None` if the user cancelled.
#[tauri::command]
pub fn import_profile_from_file(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<Option<MouseProfile>> {
    let picked = app_handle
        .dialog()
        .file()
        .add_filter("Glooblet Profile", &["json"])
        .blocking_pick_file();

    let Some(file_path) = picked else {
        return Ok(None);
    };
    let src = file_path
        .into_path()
        .map_err(|e| AppError::InvalidInput(format!("invalid file path: {e}")))?;
    let contents = std::fs::read_to_string(&src)
        .map_err(|e| AppError::InvalidInput(format!("could not read file: {e}")))?;
    let imported = import_profile(state, contents)?;
    Ok(Some(imported))
}

#[tauri::command]
pub fn list_known_games() -> Vec<&'static str> {
    calc::KNOWN_GAMES.to_vec()
}

#[tauri::command]
pub fn game_formula(game: String) -> Option<GameFormula> {
    calc::formula_for(&game)
}

#[tauri::command]
pub fn calculate_cm_per_360(
    dpi: f64,
    sensitivity: f64,
    game: String,
    custom_yaw: Option<f64>,
) -> Result<Cm360Result, String> {
    calc::cm_per_360(dpi, sensitivity, &game, custom_yaw).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn calculate_sensitivity_for_cm_per_360(
    target_cm: f64,
    dpi: f64,
    game: String,
    custom_yaw: Option<f64>,
) -> Result<Cm360Result, String> {
    calc::sensitivity_for_cm360(target_cm, dpi, &game, custom_yaw).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn compatibility_registry() -> Vec<crate::models::IntegrationStatus> {
    crate::registry::compatibility_registry()
}
