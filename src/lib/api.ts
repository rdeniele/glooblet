import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  PointerConfig,
  MouseProfile,
  SoundPreset,
  ClickSoundConfig,
  DeviceInfo,
  IntegrationStatus,
  GameFormula,
  Cm360Result,
  CapabilityReport,
} from "./types";

// A thin, typed layer over `invoke` — one function per backend command, named
// identically on both sides so the Rust source is the single source of truth
// for behavior. No business logic lives here.

// --- Mouse settings -------------------------------------------------------

export const getPointerConfig = () => invoke<PointerConfig>("get_pointer_config");

export const applyPointerConfig = (speed?: number, enhancePrecision?: boolean) =>
  invoke<{ previous: PointerConfig; applied: PointerConfig }>("apply_pointer_config", {
    speed: speed ?? null,
    enhancePrecision: enhancePrecision ?? null,
  });

export const restorePointerConfig = (config: PointerConfig) =>
  invoke<PointerConfig>("restore_pointer_config", { config });

// --- Profiles --------------------------------------------------------------

export const listProfiles = () => invoke<MouseProfile[]>("list_profiles");
export const getProfile = (id: string) => invoke<MouseProfile>("get_profile", { id });
export const createProfile = (profile: Partial<MouseProfile>) =>
  invoke<MouseProfile>("create_profile", { profile: fillProfile(profile) });
export const updateProfile = (profile: MouseProfile) =>
  invoke<MouseProfile>("update_profile", { profile });
export const deleteProfile = (id: string) => invoke<void>("delete_profile", { id });
export const duplicateProfile = (id: string, newName: string) =>
  invoke<MouseProfile>("duplicate_profile", { id, newName });
export const exportProfileToFile = (id: string, suggestedName: string) =>
  invoke<string | null>("export_profile_to_file", { id, suggestedName });
export const importProfileFromFile = () =>
  invoke<MouseProfile | null>("import_profile_from_file");

export const listKnownGames = () => invoke<string[]>("list_known_games");
export const gameFormula = (game: string) => invoke<GameFormula | null>("game_formula", { game });
export const calculateCmPer360 = (
  dpi: number,
  sensitivity: number,
  game: string,
  customYaw?: number,
) =>
  invoke<Cm360Result>("calculate_cm_per_360", {
    dpi,
    sensitivity,
    game,
    customYaw: customYaw ?? null,
  });
export const calculateSensitivityForCm360 = (
  targetCm: number,
  dpi: number,
  game: string,
  customYaw?: number,
) =>
  invoke<Cm360Result>("calculate_sensitivity_for_cm_per_360", {
    targetCm,
    dpi,
    game,
    customYaw: customYaw ?? null,
  });
export const compatibilityRegistry = () =>
  invoke<IntegrationStatus[]>("compatibility_registry");

function fillProfile(p: Partial<MouseProfile>): MouseProfile {
  return {
    id: "",
    name: p.name ?? "New profile",
    game: p.game ?? null,
    dpi: p.dpi ?? null,
    sensitivity: p.sensitivity ?? null,
    yaw_constant: p.yaw_constant ?? null,
    pointer_speed: p.pointer_speed ?? null,
    enhance_precision: p.enhance_precision ?? null,
    notes: p.notes ?? null,
    created_at: "",
    updated_at: "",
    schema_version: 0,
  };
}

// --- Sounds ------------------------------------------------------------

export const listSoundPresets = () => invoke<SoundPreset[]>("list_sound_presets");
export const previewSound = (presetId: string) =>
  invoke<void>("preview_sound", { presetId });
export const importCustomSound = (name: string, category: string) =>
  invoke<SoundPreset | null>("import_custom_sound", { name, category });
export const deleteCustomSound = (id: string) => invoke<void>("delete_custom_sound", { id });
export const getClickSoundConfig = () => invoke<ClickSoundConfig>("get_click_sound_config");
export const setClickSoundConfig = (config: ClickSoundConfig) =>
  invoke<void>("set_click_sound_config", { config });
export const setClickSoundEnabled = (enabled: boolean) =>
  invoke<void>("set_click_sound_enabled", { enabled });
export const setDesktopSoundBehavior = (behavior: AppSettings["desktop_sound_behavior"]) =>
  invoke<void>("set_desktop_sound_behavior", { behavior });

// --- Devices -----------------------------------------------------------

export const listDevices = () => invoke<DeviceInfo[]>("list_devices");

// --- Settings ------------------------------------------------------------

export const getSettings = () => invoke<AppSettings>("get_settings");
export const updateSettings = (settings: AppSettings) =>
  invoke<AppSettings>("update_settings", { settings });
export const setLaunchAtStartup = (enabled: boolean) =>
  invoke<void>("set_launch_at_startup", { enabled });
export const getLaunchAtStartup = () => invoke<boolean>("get_launch_at_startup");

// --- System ----------------------------------------------------------------

export const capabilityCheck = () => invoke<CapabilityReport>("capability_check");
export const foregroundProcessName = () => invoke<string | null>("foreground_process_name");
