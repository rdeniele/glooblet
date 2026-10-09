use crate::error::AppResult;
use crate::models::DeviceInfo;
use crate::windows_integration::devices;

#[tauri::command]
pub fn list_devices() -> AppResult<Vec<DeviceInfo>> {
    let mut found = devices::enumerate_mice()?;
    for d in &mut found {
        d.status = "Read-only".to_string();
    }
    Ok(found)
}
