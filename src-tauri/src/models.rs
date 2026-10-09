use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub launch_at_startup: bool,
    pub selected_profile_id: Option<String>,
    pub click_sound_enabled: bool,
    pub click_sound_volume: f32,
    /// "desktop_only" (default, safest) | "always" | "disabled"
    pub desktop_sound_behavior: String,
    pub first_run_complete: bool,
    pub config_version: i64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            launch_at_startup: false,
            selected_profile_id: None,
            click_sound_enabled: false,
            click_sound_volume: 0.7,
            desktop_sound_behavior: "desktop_only".into(),
            first_run_complete: false,
            config_version: SCHEMA_VERSION,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerConfig {
    /// Windows pointer speed, 1-20 (raw SPI_GETMOUSESPEED value).
    pub speed: u32,
    /// "Enhance pointer precision" — true when Windows mouse acceleration is enabled.
    pub enhance_precision: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseProfile {
    pub id: String,
    pub name: String,
    pub game: Option<String>,
    pub dpi: Option<u32>,
    pub sensitivity: Option<f64>,
    /// Optional custom degrees-per-count constant. When absent, the game's
    /// known constant (if any) is used for cm/360 calculations.
    pub yaw_constant: Option<f64>,
    pub pointer_speed: Option<u32>,
    pub enhance_precision: Option<bool>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub schema_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundPreset {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    /// Relative path inside the app's sound asset directories (built-in or user-data).
    pub file_path: String,
    pub is_builtin: bool,
    pub volume: f32,
    pub duration_ms: Option<u32>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClickSoundConfig {
    pub left_click_preset_id: Option<String>,
    pub right_click_preset_id: Option<String>,
    pub middle_click_preset_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub manufacturer: Option<String>,
    /// e.g. "USB", "Bluetooth", "PS/2", "Unknown"
    pub connection_type: String,
    pub capabilities: DeviceCapabilities,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceCapabilities {
    pub read_dpi: bool,
    pub set_dpi: bool,
    pub read_polling_rate: bool,
    pub set_polling_rate: bool,
    pub read_onboard_profile: bool,
    pub set_onboard_profile: bool,
    pub read_button_config: bool,
    pub set_button_config: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStatus {
    pub subject: String,
    pub method: String,
    pub source: String,
    pub last_reviewed: String,
    pub supported_functionality: Vec<String>,
    pub limitations: Vec<String>,
    /// "reviewed" | "limited_support" | "manual_configuration_only" | "under_review" | "disabled"
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameFormula {
    pub game: String,
    pub yaw_constant: Option<f64>,
    pub confidence: String, // "high" | "community" | "not_applicable"
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityReport {
    pub pointer_settings_readable: bool,
    pub pointer_settings_writable: bool,
    pub device_enumeration_available: bool,
    pub audio_output_available: bool,
    pub startup_registry_writable: bool,
}
