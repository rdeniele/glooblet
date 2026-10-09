//! Read-only mouse enumeration via SetupAPI (`SetupDiGetClassDevsW` +
//! `SetupDiGetDeviceRegistryPropertyW`), scoped to the Mouse device class.
//!
//! This intentionally does **not** open a raw HID handle or talk to any
//! vendor protocol: per-vendor DPI/polling-rate control (Logitech, Razer,
//! SteelSeries, ...) has no public cross-vendor API, so every device this
//! returns is reported with all `set_*` capabilities false. That's the
//! "capability-based architecture" from the spec — a device we can *see* but
//! can't *configure* is still shown, just marked manual-configuration-only,
//! rather than faking a DPI slider that does nothing.

use crate::error::AppResult;
use crate::models::{DeviceCapabilities, DeviceInfo};

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use windows::core::GUID;
    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiEnumDeviceInfo, SetupDiGetClassDevsW, SetupDiGetDeviceRegistryPropertyW,
        DIGCF_PRESENT, SPDRP_ENUMERATOR_NAME, SPDRP_FRIENDLYNAME, SPDRP_MFG, SP_DEVINFO_DATA,
    };

    // GUID_DEVCLASS_MOUSE = {4D36E96F-E325-11CE-BFC1-08002BE10318}
    const GUID_DEVCLASS_MOUSE: GUID = GUID::from_values(
        0x4D36E96F,
        0xE325,
        0x11CE,
        [0xBF, 0xC1, 0x08, 0x00, 0x2B, 0xE1, 0x03, 0x18],
    );

    fn read_string_property(
        hdevinfo: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
        devinfo: &SP_DEVINFO_DATA,
        property: windows::Win32::Devices::DeviceAndDriverInstallation::SETUP_DI_REGISTRY_PROPERTY,
    ) -> Option<String> {
        let mut buffer = [0u16; 512];
        let mut required_size: u32 = 0;
        unsafe {
            let ok = SetupDiGetDeviceRegistryPropertyW(
                hdevinfo,
                devinfo,
                property,
                None,
                Some(std::slice::from_raw_parts_mut(
                    buffer.as_mut_ptr() as *mut u8,
                    buffer.len() * 2,
                )),
                Some(&mut required_size),
            );
            if ok.is_err() {
                return None;
            }
        }
        let text = String::from_utf16_lossy(&buffer);
        let trimmed = text.trim_end_matches('\u{0}').trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn connection_type_from_enumerator(enumerator: Option<&str>) -> String {
        match enumerator.map(|s| s.to_uppercase()) {
            Some(e) if e.starts_with("USB") => "USB".into(),
            Some(e) if e.starts_with("BTH") => "Bluetooth".into(),
            Some(e) if e.starts_with("HID") => "HID".into(),
            Some(e) if e.starts_with("ACPI") || e.starts_with("ROOT") => "PS/2 or built-in".into(),
            Some(other) => other,
            None => "Unknown".into(),
        }
    }

    pub fn enumerate_mice() -> AppResult<Vec<DeviceInfo>> {
        let mut devices = Vec::new();
        unsafe {
            let hdevinfo = match SetupDiGetClassDevsW(
                Some(&GUID_DEVCLASS_MOUSE),
                None,
                None,
                DIGCF_PRESENT,
            ) {
                Ok(h) => h,
                Err(_) => return Ok(devices), // No SetupAPI access: return empty, not an error.
            };

            let mut index = 0u32;
            loop {
                let mut devinfo = SP_DEVINFO_DATA {
                    cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                    ..Default::default()
                };
                if SetupDiEnumDeviceInfo(hdevinfo, index, &mut devinfo).is_err() {
                    break;
                }

                let friendly = read_string_property(hdevinfo, &devinfo, SPDRP_FRIENDLYNAME);
                let manufacturer = read_string_property(hdevinfo, &devinfo, SPDRP_MFG);
                let enumerator = read_string_property(hdevinfo, &devinfo, SPDRP_ENUMERATOR_NAME);

                devices.push(DeviceInfo {
                    id: format!("mouse-{index}"),
                    name: friendly.unwrap_or_else(|| "Unknown mouse".to_string()),
                    manufacturer,
                    connection_type: connection_type_from_enumerator(enumerator.as_deref()),
                    capabilities: DeviceCapabilities::default(), // read-only identity; no vendor control API
                    status: "Not detected".into(),
                });

                index += 1;
            }
        }
        Ok(devices)
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;

    pub fn enumerate_mice() -> AppResult<Vec<DeviceInfo>> {
        Ok(Vec::new())
    }
}

pub use imp::enumerate_mice;
