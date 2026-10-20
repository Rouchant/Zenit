use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplaySettingsW, DISPLAY_DEVICEW, DEVMODEW, ENUM_CURRENT_SETTINGS,
};
use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSpecs {
    pub brand: String,
    pub model: String,
    pub processor: String,
    pub cores: usize,
    pub threads: usize,
    #[serde(rename = "gen")]
    pub generation: String,
    pub vendor: String,
    pub ram: String,
    pub ram_type: String,
    pub storage: String,
    pub display: String,
    pub refresh_rate: u32,
    pub gpu: String,
    pub os: String,
    pub battery: String,
    pub battery_percent: u8,
    pub is_charging: bool,
}

fn read_reg_string(hkey: HKEY, subkey: &str, value_name: &str) -> Option<String> {
    let subkey_utf16: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
    let value_name_utf16: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut opened_key: HKEY = std::ptr::null_mut();

    unsafe {
        if RegOpenKeyExW(
            hkey,
            subkey_utf16.as_ptr(),
            0,
            KEY_READ,
            &mut opened_key,
        ) != 0 {
            return None;
        }

        let mut data_type: u32 = 0;
        let mut data_size: u32 = 0;

        // First call to get size
        let status = RegQueryValueExW(
            opened_key,
            value_name_utf16.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            std::ptr::null_mut(),
            &mut data_size,
        );

        if status != 0 || data_size == 0 {
            RegCloseKey(opened_key);
            return None;
        }

        let mut buffer: Vec<u8> = vec![0; data_size as usize];
        let status = RegQueryValueExW(
            opened_key,
            value_name_utf16.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            buffer.as_mut_ptr(),
            &mut data_size,
        );

        RegCloseKey(opened_key);

        if status == 0 {
            let u16_slice = std::slice::from_raw_parts(
                buffer.as_ptr() as *const u16,
                (data_size as usize) / 2,
            );
            // Trim trailing nulls
            let trimmed: Vec<u16> = u16_slice.iter().copied().take_while(|&c| c != 0).collect();
            let os_str = OsString::from_wide(&trimmed);
            return os_str.into_string().ok().map(|s| s.trim().to_string());
        }
    }
    None
}

pub fn get_laptop_brand_and_model() -> (String, String) {
    let subkey = r"HARDWARE\DESCRIPTION\System\BIOS";
    let manufacturer = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "SystemManufacturer")
        .unwrap_or_else(|| "Generic".to_string());
    let mut model = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "SystemProductName")
        .unwrap_or_else(|| "".to_string());

    // Clean up generic OEM strings
    let clean_mfg = if manufacturer.contains("To be filled")
        || manufacturer.contains("System manufacturer")
        || manufacturer.is_empty()
    {
        read_reg_string(HKEY_LOCAL_MACHINE, subkey, "BaseBoardManufacturer")
            .unwrap_or_else(|| "Laptop".to_string())
    } else {
        manufacturer
    };

    if model.contains("To be filled") || model.contains("System Product Name") || model.is_empty() {
        model = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "BaseBoardProduct")
            .unwrap_or_else(|| "Portable PC".to_string());
    }

    let brand = clean_mfg
        .replace("ASUSTeK COMPUTER INC.", "ASUS")
        .replace("LENOVO", "Lenovo")
        .replace("Hewlett-Packard", "HP")
        .replace("Dell Inc.", "Dell")
        .replace("Micro-Star International Co., Ltd.", "MSI");

    (brand.trim().to_string(), model.trim().to_string())
}

pub fn get_display_and_hz() -> (String, u32) {
    unsafe {
        let mut devmode: DEVMODEW = std::mem::zeroed();
        devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;

        if EnumDisplaySettingsW(std::ptr::null(), ENUM_CURRENT_SETTINGS, &mut devmode) != 0 {
            let width = devmode.dmPelsWidth;
            let height = devmode.dmPelsHeight;
            let hz = devmode.dmDisplayFrequency;

            let tag = match (width, height) {
                (1920, 1080) => " (FHD)",
                (2560, 1440) => " (2K QHD)",
                (2880, 1800) => " (2.8K OLED)",
                (3840, 2160) => " (4K UHD)",
                (1366, 768) => " (HD)",
                _ => "",
            };

            return (format!("{} x {}{} @ {}Hz", width, height, tag, hz), hz);
        }
    }
    ("1920 x 1080 @ 60Hz".to_string(), 60)
}

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

pub fn get_battery_info() -> (String, u8, bool) {
    unsafe {
        let mut power_status: SYSTEM_POWER_STATUS = std::mem::zeroed();
        if GetSystemPowerStatus(&mut power_status) != 0 {
            let percent = if power_status.BatteryLifePercent <= 100 {
                power_status.BatteryLifePercent
            } else {
                100
            };
            let is_charging = power_status.ACLineStatus == 1;

            let status_str = if power_status.BatteryLifeTime != 0xFFFFFFFF {
                let hours = power_status.BatteryLifeTime / 3600;
                let mins = (power_status.BatteryLifeTime % 3600) / 60;
                if hours > 0 {
                    format!("{}% (~{}h {}m)", percent, hours, mins)
                } else {
                    format!("{}% (~{}m)", percent, mins)
                }
            } else if is_charging {
                format!("{}% (Cargando)", percent)
            } else {
                format!("{}%", percent)
            };

            return (status_str, percent, is_charging);
        }
    }
    ("Conectado AC".to_string(), 100, true)
}

fn infer_processor_details(name: &str) -> (String, String) {
    let lower = name.to_lowercase();
    let vendor = if lower.contains("intel") {
        "Intel".to_string()
    } else if lower.contains("amd") {
        "AMD".to_string()
    } else {
        "Generic".to_string()
    };

    let mut generation = String::new();
    if lower.contains("ultra") {
        generation = "Core Ultra".to_string();
    } else if let Some(pos) = lower.find("i3-")
        .or_else(|| lower.find("i5-"))
        .or_else(|| lower.find("i7-"))
        .or_else(|| lower.find("i9-"))
    {
        let slice = &lower[pos + 3..];
        let num_str: String = slice.chars().take_while(|c| c.is_ascii_digit()).collect();
        if num_str.len() >= 2 {
            let gen_num = if num_str.len() >= 4 { &num_str[..2] } else { &num_str[..1] };
            generation = format!("{}ª Gen", gen_num);
        }
    } else if lower.contains("ryzen") {
        for series in &["7000", "8000", "5000", "6000", "4000", "3000"] {
            if lower.contains(series) {
                generation = format!("Serie {}", series);
                break;
            }
        }
    } else if lower.contains("n100") || lower.contains("n200") || lower.contains("n95") {
        generation = "N-Series".to_string();
    }

    (vendor, generation)
}

pub fn detect_hardware() -> SystemSpecs {
    let mut sys = System::new_all();
    sys.refresh_all();

    let (brand, model) = get_laptop_brand_and_model();
    let (display, refresh_rate) = get_display_and_hz();
    let gpu = get_gpu_name();
    let (battery, battery_percent, is_charging) = get_battery_info();

    // CPU info
    let cpu_name = sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_else(|| "Procesador".to_string());
    let cores = sys.physical_core_count().unwrap_or(sys.cpus().len());
    let threads = sys.cpus().len();
    let (vendor, generation) = infer_processor_details(&cpu_name);

    // RAM info
    let total_ram_bytes = sys.total_memory();
    let ram_gb = (total_ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u64;
    let ram = format!("{} GB", ram_gb);
    let ram_type = if cpu_name.contains("Ultra") || cpu_name.contains("8000") || cpu_name.contains("7000") {
        "DDR5"
    } else {
        "DDR4"
    }.to_string();

    // Storage info
    let disks = Disks::new_with_refreshed_list();
    let mut total_disk_bytes: u64 = 0;
    for disk in &disks {
        if disk.is_removable() == false {
            total_disk_bytes += disk.total_space();
        }
    }
    let total_gb = (total_disk_bytes as f64 / 1_000_000_000.0).round() as u64;
    let storage = if total_gb >= 900 {
        "1TB NVMe SSD".to_string()
    } else if total_gb >= 450 {
        "512GB SSD".to_string()
    } else if total_gb >= 220 {
        "256GB SSD".to_string()
    } else if total_gb >= 110 {
        "128GB SSD".to_string()
    } else {
        format!("{}GB SSD", total_gb)
    };

    // OS info
    let os_name = System::name().unwrap_or_else(|| "Windows".to_string());
    let os_version = System::os_version().unwrap_or_default();
    let os = if os_name.contains("Windows") {
        if os_version.starts_with("11") || os_version.contains("22") || os_version.contains("23") || os_version.contains("24") {
            "Windows 11 Home".to_string()
        } else {
            "Windows 10 Home".to_string()
        }
    } else {
        format!("{} {}", os_name, os_version)
    };

    SystemSpecs {
        brand,
        model,
        processor: cpu_name,
        cores,
        threads,
        generation,
        vendor,
        ram,
        ram_type,
        storage,
        display,
        refresh_rate,
        gpu,
        os,
        battery,
        battery_percent,
        is_charging,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hardware_execution() {
        let specs = detect_hardware();
        assert!(!specs.processor.is_empty(), "Processor should not be empty");
        assert!(specs.cores > 0, "Cores should be at least 1");
        assert!(specs.threads > 0, "Threads should be at least 1");
        assert!(!specs.ram.is_empty(), "RAM should not be empty");
        assert!(!specs.os.is_empty(), "OS should not be empty");
    }

    #[test]
    fn test_ram_formatting() {
        let specs = detect_hardware();
        assert!(specs.ram.contains("GB"), "RAM description should contain GB");
    }
}
