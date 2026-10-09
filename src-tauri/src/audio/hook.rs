//! Optional, off-by-default global mouse-click listener for Click Studio.
//!
//! This uses the documented `WH_MOUSE_LL` low-level hook purely to *observe*
//! button-down events so a sound can be played — it never modifies, delays,
//! or swallows an event. `CallNextHookEx` is always invoked, unconditionally
//! and first in the control-flow sense (the hook never returns non-zero), so
//! the real click always reaches the game/app exactly as if Glooblet weren't
//! running. Nothing here reads game memory, injects into another process, or
//! touches raw HID/XInput input — it is a desktop accessibility-style hook,
//! the same mechanism screen readers and clicker utilities use.
//!
//! Safety-critical rules for anything added to this file:
//! 1. The callback must return in well under the OS's low-level-hook timeout
//!    (default ~300ms) or Windows silently unhooks it. No blocking I/O, no
//!    waiting locks, no heap-heavy work on the hot path.
//! 2. `CallNextHookEx` must run no matter what — wrap our own logic in
//!    `catch_unwind` so a panic can't unwind across the `extern "system"` FFI
//!    boundary (that's undefined behavior) and can't skip the forward call.
//! 3. Only `*_DOWN` messages are inspected; we never consume or alter input.

use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Default)]
pub struct HookConfig {
    pub enabled: bool,
    pub desktop_only: bool,
    pub left: Option<(PathBuf, f32)>,
    pub right: Option<(PathBuf, f32)>,
    pub middle: Option<(PathBuf, f32)>,
    pub known_game_exes: Vec<String>,
}

static HOOK_CONFIG: OnceLock<Mutex<HookConfig>> = OnceLock::new();

fn config_cell() -> &'static Mutex<HookConfig> {
    HOOK_CONFIG.get_or_init(|| Mutex::new(HookConfig::default()))
}

pub fn update_config(cfg: HookConfig) {
    if let Ok(mut guard) = config_cell().lock() {
        *guard = cfg;
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use crate::audio::AudioEngine;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
        TranslateMessage, UnhookWindowsHookEx, HHOOK, HOOKPROC, MSG, WH_MOUSE_LL, WM_LBUTTONDOWN,
        WM_MBUTTONDOWN, WM_QUIT, WM_RBUTTONDOWN,
    };

    // The audio engine the hook plays through. Set once when the hook thread
    // starts; the hook thread outlives every click it handles so this is safe.
    static AUDIO: OnceLock<Arc<AudioEngine>> = OnceLock::new();
    static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

    unsafe extern "system" fn low_level_mouse_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // Rule 2: never let a panic cross this boundary.
        let _ = std::panic::catch_unwind(|| {
            if code == 0 {
                handle_click(wparam.0 as u32);
            }
        });
        // Rule 3: always forward, unmodified.
        CallNextHookEx(HHOOK::default(), code, wparam, lparam)
    }

    fn handle_click(message: u32) {
        let Some(cfg) = config_cell().lock().ok().map(|g| g.clone()) else { return };
        if !cfg.enabled {
            return;
        }

        let slot = match message {
            m if m == WM_LBUTTONDOWN => &cfg.left,
            m if m == WM_RBUTTONDOWN => &cfg.right,
            m if m == WM_MBUTTONDOWN => &cfg.middle,
            _ => return,
        };
        let Some((path, volume)) = slot else { return };

        if cfg.desktop_only && crate::windows_integration::foreground::is_known_game_foreground(&cfg.known_game_exes) {
            return;
        }

        if let Some(engine) = AUDIO.get() {
            engine.play(path, *volume);
        }
    }

    pub struct HookHandle {
        thread: Option<std::thread::JoinHandle<()>>,
    }

    impl HookHandle {
        pub fn stop(mut self) {
            let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
            if tid != 0 {
                unsafe {
                    let _ = PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
                }
            }
            if let Some(t) = self.thread.take() {
                let _ = t.join();
            }
        }
    }

    pub fn start(engine: Arc<AudioEngine>) -> AppResult<HookHandle> {
        AUDIO.get_or_init(|| engine);

        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<AppResult<()>>();

        let thread = std::thread::spawn(move || unsafe {
            HOOK_THREAD_ID.store(windows::Win32::System::Threading::GetCurrentThreadId(), Ordering::SeqCst);

            let proc: HOOKPROC = Some(low_level_mouse_proc);
            let hook = match SetWindowsHookExW(WH_MOUSE_LL, proc, None, 0) {
                Ok(h) => h,
                Err(e) => {
                    let _ = ready_tx.send(Err(AppError::WindowsApi(format!(
                        "failed to install mouse hook: {e}"
                    ))));
                    return;
                }
            };
            let _ = ready_tx.send(Ok(()));

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
        });

        ready_rx
            .recv()
            .map_err(|_| AppError::WindowsApi("mouse hook thread failed to start".into()))??;

        Ok(HookHandle { thread: Some(thread) })
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;
    use crate::audio::AudioEngine;
    use std::sync::Arc;

    pub struct HookHandle;
    impl HookHandle {
        pub fn stop(self) {}
    }

    pub fn start(_engine: Arc<AudioEngine>) -> AppResult<HookHandle> {
        Err(AppError::Unsupported(
            "global click monitoring is Windows-only".into(),
        ))
    }
}

pub use imp::{start, HookHandle};
