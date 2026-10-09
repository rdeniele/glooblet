use crate::audio::hook::{self, HookConfig};
use crate::audio::validate_audio_file;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{ClickSoundConfig, SoundPreset};
use crate::state::AppState;
use crate::{db::save_settings, seed};
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

fn resolve_sound_path(state: &AppState, preset: &SoundPreset) -> std::path::PathBuf {
    if preset.is_builtin {
        state.builtin_sounds_dir().join(&preset.file_path)
    } else {
        state.custom_sounds_dir().join(&preset.file_path)
    }
}

#[tauri::command]
pub fn list_sound_presets(state: State<AppState>) -> AppResult<Vec<SoundPreset>> {
    let conn = state.db.lock().unwrap();
    db::sounds::list(&conn)
}

#[tauri::command]
pub fn preview_sound(state: State<AppState>, preset_id: String) -> AppResult<()> {
    let (preset, master_volume) = {
        let conn = state.db.lock().unwrap();
        let preset = db::sounds::get(&conn, &preset_id)?;
        let settings = db::load_settings(&conn)?;
        (preset, settings.click_sound_volume)
    };
    let path = resolve_sound_path(&state, &preset);
    if !path.exists() {
        return Err(AppError::NotFound(format!(
            "sound file for preset \"{}\" is missing on disk",
            preset.name
        )));
    }
    state.audio.play(&path, effective_volume(preset.volume, master_volume));
    Ok(())
}

/// Combines a preset's own level with the master volume slider — the single
/// place both preview and real click playback compute "how loud," so the
/// master slider actually affects what you hear instead of being stored and
/// ignored.
fn effective_volume(preset_volume: f32, master_volume: f32) -> f32 {
    (preset_volume * master_volume).clamp(0.0, 1.0)
}

/// Opens a native "open file" dialog scoped to audio extensions, then
/// validates and copies whatever the user picked into app-controlled
/// storage. The backend owns the dialog call so the source path always comes
/// from the OS picker the user just interacted with, never from a path the
/// frontend hands in on its own.
#[tauri::command]
pub fn import_custom_sound(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    name: String,
    category: String,
) -> AppResult<Option<SoundPreset>> {
    let picked = app_handle
        .dialog()
        .file()
        .add_filter("Audio", &["wav", "mp3", "ogg"])
        .blocking_pick_file();

    let Some(file_path) = picked else {
        return Ok(None);
    };
    let src = file_path
        .into_path()
        .map_err(|e| AppError::InvalidInput(format!("invalid file path: {e}")))?;
    let validated = validate_audio_file(&src)?;

    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("wav")
        .to_lowercase();
    let stored_name = format!("{}.{}", Uuid::new_v4(), ext);

    let dest_dir = state.custom_sounds_dir();
    std::fs::create_dir_all(&dest_dir)?;
    let dest_path = dest_dir.join(&stored_name);
    std::fs::copy(src, &dest_path)?;

    let conn = state.db.lock().unwrap();
    let preset = db::sounds::insert(
        &conn,
        SoundPreset {
            id: String::new(),
            name,
            category,
            description: "Imported sound".into(),
            file_path: stored_name,
            is_builtin: false,
            volume: 0.7,
            duration_ms: validated.duration_ms,
            created_at: String::new(),
        },
    )?;
    Ok(Some(preset))
}

#[tauri::command]
pub fn delete_custom_sound(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    let preset = db::sounds::delete_custom(&conn, &id)?;
    let path = state.custom_sounds_dir().join(&preset.file_path);
    let _ = std::fs::remove_file(path); // best-effort; DB row is already gone
    Ok(())
}

#[tauri::command]
pub fn get_click_sound_config(state: State<AppState>) -> AppResult<ClickSoundConfig> {
    let conn = state.db.lock().unwrap();
    db::sounds::get_click_config(&conn)
}

#[tauri::command]
pub fn set_click_sound_config(
    state: State<AppState>,
    config: ClickSoundConfig,
) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    db::sounds::save_click_config(&conn, &config)?;
    drop(conn);
    sync_hook_config(&state)
}

/// Re-reads settings + click config + presets from the DB and pushes a fresh
/// snapshot into the (possibly running) mouse hook. Called after any change
/// that could affect what the hook should play.
pub fn sync_hook_config(state: &AppState) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    let settings = db::load_settings(&conn)?;
    let click_cfg = db::sounds::get_click_config(&conn)?;
    let presets = db::sounds::list(&conn)?;
    drop(conn);

    let master_volume = settings.click_sound_volume;
    let find_path_volume = |id: &Option<String>| -> Option<(std::path::PathBuf, f32)> {
        let id = id.as_ref()?;
        let preset = presets.iter().find(|p| &p.id == id)?;
        Some((resolve_sound_path(state, preset), effective_volume(preset.volume, master_volume)))
    };

    hook::update_config(HookConfig {
        enabled: settings.click_sound_enabled,
        desktop_only: settings.desktop_sound_behavior == "desktop_only",
        left: find_path_volume(&click_cfg.left_click_preset_id),
        right: find_path_volume(&click_cfg.right_click_preset_id),
        middle: find_path_volume(&click_cfg.middle_click_preset_id),
        known_game_exes: seed::known_game_executables(),
    });
    Ok(())
}

/// Master enable/disable for Click Studio. Starts or stops the OS-level hook
/// accordingly; stopping always wins immediately (the hook thread is joined
/// before this returns, and `AudioEngine::stop_all` silences anything
/// already queued).
#[tauri::command]
pub fn set_click_sound_enabled(state: State<AppState>, enabled: bool) -> AppResult<()> {
    {
        let conn = state.db.lock().unwrap();
        let mut settings = db::load_settings(&conn)?;
        settings.click_sound_enabled = enabled;
        save_settings(&conn, &settings)?;
    }
    sync_hook_config(&state)?;

    let mut hook_guard = state.hook.lock().unwrap();
    if enabled {
        if hook_guard.is_none() {
            let handle = hook::start(state.audio.clone())?;
            *hook_guard = Some(handle);
        }
    } else {
        state.audio.stop_all();
        if let Some(h) = hook_guard.take() {
            h.stop();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_desktop_sound_behavior(state: State<AppState>, behavior: String) -> AppResult<()> {
    if !["desktop_only", "always", "disabled"].contains(&behavior.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "unknown desktop sound behavior \"{behavior}\""
        )));
    }
    {
        let conn = state.db.lock().unwrap();
        let mut settings = db::load_settings(&conn)?;
        settings.desktop_sound_behavior = behavior;
        save_settings(&conn, &settings)?;
    }
    sync_hook_config(&state)
}
