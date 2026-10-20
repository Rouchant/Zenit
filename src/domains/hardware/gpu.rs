use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows_sys::Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW};

pub fn get_gpu_name() -> String {
    unsafe {
        let mut dev: DISPLAY_DEVICEW = std::mem::zeroed();
        dev.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

        let mut index = 0;
        let mut primary_gpu = String::new();
        let mut secondary_gpu = String::new();

        while EnumDisplayDevicesW(std::ptr::null(), index, &mut dev, 0) != 0 {
            let name_slice = &dev.DeviceString;
            let end_idx = name_slice.iter().position(|&c| c == 0).unwrap_or(name_slice.len());
            let os_str = OsString::from_wide(&name_slice[..end_idx]);
            if let Ok(name) = os_str.into_string() {
                let trimmed = name.trim().to_string();
                if !trimmed.is_empty() {
                    if trimmed.contains("NVIDIA") || trimmed.contains("GeForce") || trimmed.contains("Radeon RX") {
                        primary_gpu = trimmed;
                        break;
                    } else if secondary_gpu.is_empty() {
                        secondary_gpu = trimmed;
                    }
                }
            }
            index += 1;
            dev.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
        }

        if !primary_gpu.is_empty() {
            return primary_gpu;
        }
        if !secondary_gpu.is_empty() {
            return secondary_gpu;
        }
    }
    "Gráficos Integrados".to_string()
}
