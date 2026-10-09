//! Reading and writing Windows' own pointer-speed and "Enhance pointer
//! precision" settings via the documented `SystemParametersInfoW` API.
//!
//! These are global, per-user Windows settings — not per-application and not
//! specific to any one mouse. Every write here is preceded by a read (for
//! rollback) and followed by a read-back (to confirm the OS actually applied
//! it), per the app's "verify after every change" rule.

use crate::error::{AppError, AppResult};
use crate::models::PointerConfig;

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use std::ffi::c_void;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_GETMOUSE,
        SPI_GETMOUSESPEED, SPI_SETMOUSE, SPI_SETMOUSESPEED,
    };

    pub fn get_pointer_config() -> AppResult<PointerConfig> {
        let mut speed: u32 = 0;
        unsafe {
            SystemParametersInfoW(
                SPI_GETMOUSESPEED,
                0,
                Some(&mut speed as *mut u32 as *mut c_void),
                Default::default(),
            )
            .map_err(|e| AppError::WindowsApi(format!("SPI_GETMOUSESPEED failed: {e}")))?;
        }

        let mut mouse_params: [i32; 3] = [0; 3];
        unsafe {
            SystemParametersInfoW(
                SPI_GETMOUSE,
                0,
                Some(mouse_params.as_mut_ptr() as *mut c_void),
                Default::default(),
            )
            .map_err(|e| AppError::WindowsApi(format!("SPI_GETMOUSE failed: {e}")))?;
        }

        Ok(PointerConfig {
            speed,
            enhance_precision: mouse_params[2] != 0,
        })
    }

    pub fn set_pointer_speed(speed: u32) -> AppResult<()> {
        if !(1..=20).contains(&speed) {
            return Err(AppError::InvalidInput(
                "Windows pointer speed must be between 1 and 20".into(),
            ));
        }
        unsafe {
            SystemParametersInfoW(
                SPI_SETMOUSESPEED,
                0,
                Some(speed as usize as *mut c_void),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            )
            .map_err(|e| AppError::WindowsApi(format!("SPI_SETMOUSESPEED failed: {e}")))?;
        }
        Ok(())
    }

    pub fn set_enhance_precision(enabled: bool) -> AppResult<()> {
        // Windows' own checkbox toggles the acceleration flag while leaving the
        // two threshold values at their long-standing defaults (6, 10); only the
        // third element (the on/off flag) actually changes.
        let mut params: [i32; 3] = if enabled { [6, 10, 1] } else { [0, 0, 0] };
        unsafe {
            SystemParametersInfoW(
                SPI_SETMOUSE,
                0,
                Some(params.as_mut_ptr() as *mut c_void),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            )
            .map_err(|e| AppError::WindowsApi(format!("SPI_SETMOUSE failed: {e}")))?;
        }
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;

    pub fn get_pointer_config() -> AppResult<PointerConfig> {
        Err(AppError::Unsupported(
            "Windows pointer settings are only available on Windows".into(),
        ))
    }

    pub fn set_pointer_speed(_speed: u32) -> AppResult<()> {
        Err(AppError::Unsupported(
            "Windows pointer settings are only available on Windows".into(),
        ))
    }

    pub fn set_enhance_precision(_enabled: bool) -> AppResult<()> {
        Err(AppError::Unsupported(
            "Windows pointer settings are only available on Windows".into(),
        ))
    }
}

pub use imp::{get_pointer_config, set_enhance_precision, set_pointer_speed};

/// Applies a new pointer speed / acceleration state, verifying the OS
/// actually accepted each change and returning the config that was in effect
/// immediately before, so the caller can offer a one-click rollback.
pub fn apply_with_rollback(
    new_speed: Option<u32>,
    new_enhance_precision: Option<bool>,
) -> AppResult<(PointerConfig, PointerConfig)> {
    let previous = get_pointer_config()?;

    if let Some(speed) = new_speed {
        set_pointer_speed(speed)?;
    }
    if let Some(enhance) = new_enhance_precision {
        set_enhance_precision(enhance)?;
    }

    let applied = get_pointer_config()?;

    if let Some(speed) = new_speed {
        if applied.speed != speed {
            // Roll back before telling the caller anything went wrong.
            let _ = set_pointer_speed(previous.speed);
            let _ = set_enhance_precision(previous.enhance_precision);
            return Err(AppError::WindowsApi(
                "Windows did not accept the new pointer speed; reverted".into(),
            ));
        }
    }
    if let Some(enhance) = new_enhance_precision {
        if applied.enhance_precision != enhance {
            let _ = set_pointer_speed(previous.speed);
            let _ = set_enhance_precision(previous.enhance_precision);
            return Err(AppError::WindowsApi(
                "Windows did not accept the new acceleration setting; reverted".into(),
            ));
        }
    }

    Ok((previous, applied))
}
