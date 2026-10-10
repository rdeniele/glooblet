pub mod audio;
pub mod calc;
pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod registry;
pub mod seed;
pub mod state;
pub mod windows_integration;

use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Builds everything `setup()` needs, as a `Result` instead of a chain of
/// `.expect()`s. Startup used to panic on the first failure here — and with
/// `panic = "abort"` in the release profile plus the windowed subsystem
/// hiding the console, that meant the app could vanish with zero explanation
/// the instant it launched (a disk-full app-data dir, a locked database file
/// from a previous crashed instance, a broken install). Collecting failures
/// into a `Result` lets `run()` show the user an actual message instead.
fn build_app_state(app: &tauri::App) -> Result<AppState, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not find a place to store app data: {e}"))?;

    // `resource_dir()` alone points at different places in dev vs. a
    // packaged build; `resolve(.., BaseDirectory::Resource)` is the API
    // Tauri actually intends for finding a specific bundled file in both
    // modes, so bundled resources are located through it rather than a
    // hand-rolled `resource_dir().join(...)`.
    let builtin_sounds_dir = app
        .path()
        .resolve("resources/sounds", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Could not locate the bundled sound files: {e}"))?;

    let conn = db::open(&app_data_dir)
        .map_err(|e| format!("Could not open the local database at {app_data_dir:?}: {e}"))?;
    seed::seed_builtin_sounds(&conn)
        .map_err(|e| format!("Could not set up the built-in sound library: {e}"))?;

    let audio = Arc::new(audio::AudioEngine::new());

    let app_state = AppState {
        db: Mutex::new(conn),
        audio: audio.clone(),
        hook: Mutex::new(None),
        app_data_dir,
        builtin_sounds_dir,
    };

    // If click sounds were left enabled from a previous session, restore the
    // hook on launch so behavior survives a restart. A failure here (e.g. the
    // hook API being unavailable) shouldn't block the rest of the app from
    // starting, so it's logged rather than propagated.
    {
        let conn = app_state.db.lock().unwrap();
        match db::load_settings(&conn) {
            Ok(settings) if settings.click_sound_enabled => {
                drop(conn);
                if let Ok(handle) = audio::hook::start(audio.clone()) {
                    *app_state.hook.lock().unwrap() = Some(handle);
                } else {
                    log::warn!("could not restore the click-sound hook on startup");
                }
            }
            Ok(_) => {}
            Err(e) => log::warn!("could not load settings on startup: {e}"),
        }
    }

    Ok(app_state)
}

/// Shows a native "Glooblet failed to start" message box and exits. Used
/// only for the handful of startup failures that leave the app with nothing
/// useful to show — deliberately not a panic, so it never hits the
/// `panic = "abort"` release profile's silent, unwind-free termination.
fn fatal_startup_error(message: &str) -> ! {
    #[cfg(target_os = "windows")]
    {
        use windows::core::PCWSTR;
        use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
        let text: Vec<u16> = format!("Glooblet couldn't start:\n\n{message}")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let title: Vec<u16> = "Glooblet".encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(text.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONERROR,
            );
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("Glooblet couldn't start: {message}");
    }
    std::process::exit(1)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_state = match build_app_state(app) {
                Ok(state) => state,
                Err(message) => fatal_startup_error(&message),
            };

            app.manage(app_state);

            let state = app.state::<AppState>();
            let _ = commands::sounds::sync_hook_config(&state);

            Ok(())
        })
        .on_window_event(|window, event| {
            // Stop background audio/hook work the moment the window closes, so
            // nothing keeps running once the user has visibly quit the app.
            if let tauri::WindowEvent::Destroyed = event {
                if let Some(state) = window.try_state::<AppState>() {
                    state.audio.stop_all();
                    if let Some(hook) = state.hook.lock().unwrap().take() {
                        hook.stop();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::mouse_settings::get_pointer_config,
            commands::mouse_settings::apply_pointer_config,
            commands::mouse_settings::restore_pointer_config,
            commands::profiles::list_profiles,
            commands::profiles::get_profile,
            commands::profiles::create_profile,
            commands::profiles::update_profile,
            commands::profiles::delete_profile,
            commands::profiles::duplicate_profile,
            commands::profiles::export_profile,
            commands::profiles::import_profile,
            commands::profiles::export_profile_to_file,
            commands::profiles::import_profile_from_file,
            commands::profiles::list_known_games,
            commands::profiles::game_formula,
            commands::profiles::calculate_cm_per_360,
            commands::profiles::calculate_sensitivity_for_cm_per_360,
            commands::profiles::compatibility_registry,
            commands::sounds::list_sound_presets,
            commands::sounds::preview_sound,
            commands::sounds::import_custom_sound,
            commands::sounds::delete_custom_sound,
            commands::sounds::get_click_sound_config,
            commands::sounds::set_click_sound_config,
            commands::sounds::set_click_sound_enabled,
            commands::sounds::set_desktop_sound_behavior,
            commands::devices::list_devices,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::set_launch_at_startup,
            commands::settings::get_launch_at_startup,
            commands::system::capability_check,
            commands::system::foreground_process_name,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
