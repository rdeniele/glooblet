use crate::audio::hook::HookHandle;
use crate::audio::AudioEngine;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub audio: Arc<AudioEngine>,
    pub hook: Mutex<Option<HookHandle>>,
    pub app_data_dir: PathBuf,
    /// Resolved once at startup via Tauri's own resource resolver
    /// (`BaseDirectory::Resource`), which is the only reliable way to find
    /// bundled files — `resource_dir()` alone points somewhere different in
    /// dev (`target/debug`) than in a packaged build, and manually
    /// re-joining the relative resource path on top of it guesses wrong in
    /// one of the two modes.
    pub builtin_sounds_dir: PathBuf,
}

impl AppState {
    pub fn custom_sounds_dir(&self) -> PathBuf {
        self.app_data_dir.join("sounds").join("custom")
    }

    pub fn builtin_sounds_dir(&self) -> PathBuf {
        self.builtin_sounds_dir.clone()
    }
}
