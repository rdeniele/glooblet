// Mirrors src-tauri/src/models.rs — kept in sync by hand since both sides are
// small and change together; see that file for field-level rationale.

export interface AppSettings {
  theme: string;
  launch_at_startup: boolean;
  selected_profile_id: string | null;
  click_sound_enabled: boolean;
  click_sound_volume: number;
  desktop_sound_behavior: "desktop_only" | "always" | "disabled";
  first_run_complete: boolean;
  config_version: number;
}

export interface PointerConfig {
  speed: number;
  enhance_precision: boolean;
}

export interface MouseProfile {
  id: string;
  name: string;
  game: string | null;
  dpi: number | null;
  sensitivity: number | null;
  yaw_constant: number | null;
  pointer_speed: number | null;
  enhance_precision: boolean | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
  schema_version: number;
}

export interface SoundPreset {
  id: string;
  name: string;
  category: string;
  description: string;
  file_path: string;
  is_builtin: boolean;
  volume: number;
  duration_ms: number | null;
  created_at: string;
}

export interface ClickSoundConfig {
  left_click_preset_id: string | null;
  right_click_preset_id: string | null;
  middle_click_preset_id: string | null;
}

export interface DeviceCapabilities {
  read_dpi: boolean;
  set_dpi: boolean;
  read_polling_rate: boolean;
  set_polling_rate: boolean;
  read_onboard_profile: boolean;
  set_onboard_profile: boolean;
  read_button_config: boolean;
  set_button_config: boolean;
}

export interface DeviceInfo {
  id: string;
  name: string;
  manufacturer: string | null;
  connection_type: string;
  capabilities: DeviceCapabilities;
  status: string;
}

export interface IntegrationStatus {
  subject: string;
  method: string;
  source: string;
  last_reviewed: string;
  supported_functionality: string[];
  limitations: string[];
  status:
    | "reviewed"
    | "limited_support"
    | "manual_configuration_only"
    | "under_review"
    | "disabled";
}

export interface GameFormula {
  game: string;
  yaw_constant: number | null;
  confidence: "high" | "community" | "not_applicable";
  note: string;
}

export interface Cm360Result {
  cm_per_360: number;
  yaw_constant_used: number;
  source: "game_default" | "custom_override";
  resulting_sensitivity: number | null;
}

export interface CapabilityReport {
  pointer_settings_readable: boolean;
  pointer_settings_writable: boolean;
  device_enumeration_available: boolean;
  audio_output_available: boolean;
  startup_registry_writable: boolean;
}

export interface ErrorPayload {
  code: string;
  message: string;
}

export function isErrorPayload(e: unknown): e is ErrorPayload {
  return (
    typeof e === "object" &&
    e !== null &&
    "code" in e &&
    "message" in e
  );
}
