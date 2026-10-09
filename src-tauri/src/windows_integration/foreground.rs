//! Read-only foreground-window → process-name lookup, built entirely from
//! documented Win32 APIs (`GetForegroundWindow`, `QueryFullProcessImageNameW`).
//!
//! This never reads or writes game memory and never attaches to another
//! process beyond asking Windows for its image path. It backs two things:
//! Click Studio's "don't play click sounds over a game" default, and letting
//! Devices/Overview show what's currently focused. It does **not** drive
//! automatic game-profile switching — that stays a manual, explicit action.

use crate::error::AppResult;

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    pub fn foreground_process_name() -> AppResult<Option<String>> {
        unsafe {
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.0.is_null() {
                return Ok(None);
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return Ok(None);
            }

            let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(h) => h,
                Err(_) => return Ok(None), // e.g. a protected/elevated process we can't inspect
            };

            let mut buffer = [0u16; MAX_PATH as usize];
            let mut size = buffer.len() as u32;
            let ok = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                windows::core::PWSTR(buffer.as_mut_ptr()),
                &mut size,
            );
            let _ = CloseHandle(handle);

            if ok.is_err() || size == 0 {
                return Ok(None);
            }

            let path = String::from_utf16_lossy(&buffer[..size as usize]);
            let exe_name = path
                .rsplit(['\\', '/'])
                .next()
                .unwrap_or(&path)
                .to_string();
            Ok(Some(exe_name))
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;

    pub fn foreground_process_name() -> AppResult<Option<String>> {
        Ok(None)
    }
}

pub use imp::foreground_process_name;

/// Case-insensitive match against a list of known executable names
/// (e.g. `["cs2.exe", "valorant.exe"]`) used to suppress desktop click
/// sounds while a recognized game has focus.
pub fn is_known_game_foreground(known_exe_names: &[String]) -> bool {
    match foreground_process_name() {
        Ok(Some(name)) => known_exe_names
            .iter()
            .any(|known| known.eq_ignore_ascii_case(&name)),
        _ => false,
    }
}
