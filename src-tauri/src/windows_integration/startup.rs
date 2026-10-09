//! "Launch at Windows startup" via the per-user Run registry key
//! (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`). This is the
//! standard, documented, non-admin mechanism for an app to start itself with
//! the user's session — no scheduled task, no service, no elevation.

use crate::error::AppResult;

const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Glooblet";

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use crate::error::AppError;
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
        HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
    };

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn open_key(write: bool) -> AppResult<HKEY> {
        let path = to_wide(RUN_KEY_PATH);
        let mut hkey = HKEY::default();
        let access = if write { KEY_WRITE } else { KEY_READ };
        unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                0,
                access,
                &mut hkey,
            )
            .ok()
            .map_err(|e| AppError::WindowsApi(format!("failed to open Run key: {e}")))?;
        }
        Ok(hkey)
    }

    pub fn is_enabled() -> AppResult<bool> {
        let hkey = open_key(false)?;
        let name = to_wide(VALUE_NAME);
        let mut data_type = REG_SZ;
        let result = unsafe {
            RegQueryValueExW(
                hkey,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut data_type),
                None,
                None,
            )
        };
        unsafe {
            let _ = RegCloseKey(hkey);
        }
        Ok(result.is_ok())
    }

    pub fn set_enabled(enabled: bool, exe_path: &str) -> AppResult<()> {
        let hkey = open_key(true)?;
        let name = to_wide(VALUE_NAME);

        let result = if enabled {
            let value = to_wide(exe_path);
            let bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(value.as_ptr() as *const u8, value.len() * 2)
            };
            unsafe { RegSetValueExW(hkey, PCWSTR(name.as_ptr()), 0, REG_SZ, Some(bytes)) }
        } else {
            unsafe { RegDeleteValueW(hkey, PCWSTR(name.as_ptr())) }
        };

        unsafe {
            let _ = RegCloseKey(hkey);
        }

        // Deleting a value that's already absent is not an error for us.
        if enabled {
            result
                .ok()
                .map_err(|e| AppError::WindowsApi(format!("failed to write Run key: {e}")))?;
        }
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;
    use crate::error::AppError;

    pub fn is_enabled() -> AppResult<bool> {
        Err(AppError::Unsupported("startup registry is Windows-only".into()))
    }

    pub fn set_enabled(_enabled: bool, _exe_path: &str) -> AppResult<()> {
        Err(AppError::Unsupported("startup registry is Windows-only".into()))
    }
}

pub use imp::{is_enabled, set_enabled};
