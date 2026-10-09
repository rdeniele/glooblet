use crate::commands::sounds::sync_hook_config;
use crate::db;
use crate::error::AppResult;
use crate::models::AppSettings;
use crate::state::AppState;
use crate::windows_integration::startup;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppResult<AppSettings> {
    let conn = state.db.lock().unwrap();
    db::load_settings(&conn)
}

#[tauri::command]
pub fn update_settings(state: State<AppState>, settings: AppSettings) -> AppResult<AppSettings> {
    {
        let conn = state.db.lock().unwrap();
        db::save_settings(&conn, &settings)?;
    }
    sync_hook_config(&state)?;
    Ok(settings)
}

#[tauri::command]
pub fn set_launch_at_startup(state: State<AppState>, enabled: bool) -> AppResult<()> {
    let exe_path = std::env::current_exe()?;
    startup::set_enabled(enabled, &exe_path.to_string_lossy())?;

    let conn = state.db.lock().unwrap();
    let mut settings = db::load_settings(&conn)?;
    settings.launch_at_startup = enabled;
    db::save_settings(&conn, &settings)?;
    Ok(())
}

#[tauri::command]
pub fn get_launch_at_startup() -> AppResult<bool> {
    startup::is_enabled()
}
