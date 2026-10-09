use crate::error::AppResult;
use crate::models::PointerConfig;
use crate::windows_integration::pointer;

#[tauri::command]
pub fn get_pointer_config() -> AppResult<PointerConfig> {
    pointer::get_pointer_config()
}

#[derive(Debug, serde::Serialize)]
pub struct ApplyPointerResult {
    pub previous: PointerConfig,
    pub applied: PointerConfig,
}

/// Applies a pointer-speed and/or acceleration change, verifying it took
/// effect and automatically rolling back if Windows didn't accept it.
#[tauri::command]
pub fn apply_pointer_config(
    speed: Option<u32>,
    enhance_precision: Option<bool>,
) -> AppResult<ApplyPointerResult> {
    let (previous, applied) = pointer::apply_with_rollback(speed, enhance_precision)?;
    Ok(ApplyPointerResult { previous, applied })
}

/// Explicit rollback action for the "restore previous configuration" button.
#[tauri::command]
pub fn restore_pointer_config(config: PointerConfig) -> AppResult<PointerConfig> {
    pointer::set_pointer_speed(config.speed)?;
    pointer::set_enhance_precision(config.enhance_precision)?;
    pointer::get_pointer_config()
}
