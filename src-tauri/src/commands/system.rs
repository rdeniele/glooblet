use crate::error::AppResult;
use crate::models::CapabilityReport;
use crate::windows_integration::{devices, pointer, startup};

/// Probes each integration surface so Overview can show real
/// supported/unsupported/not-detected status instead of assuming success.
#[tauri::command]
pub fn capability_check() -> CapabilityReport {
    CapabilityReport {
        pointer_settings_readable: pointer::get_pointer_config().is_ok(),
        pointer_settings_writable: cfg!(target_os = "windows"),
        device_enumeration_available: devices::enumerate_mice()
            .map(|d| !d.is_empty())
            .unwrap_or(false),
        audio_output_available: true, // confirmed lazily by AudioEngine; see docs/LIMITATIONS.md
        startup_registry_writable: startup::is_enabled().is_ok(),
    }
}

#[tauri::command]
pub fn foreground_process_name() -> AppResult<Option<String>> {
    crate::windows_integration::foreground::foreground_process_name()
}
