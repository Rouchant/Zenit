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
        .unwrap_or_else(|| "ASUS".to_string());
    let mut model = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "SystemProductName")
        .unwrap_or_else(|| "".to_string());

    let is_oem_placeholder = |s: &str| -> bool {
        let l = s.to_lowercase();
        l.contains("to be filled")
            || l.contains("system manufacturer")
            || l.contains("system product name")
            || l.contains("o.e.m.")
            || l.contains("default string")
            || l.trim().is_empty()
    };

    // Clean up generic OEM strings
    let mut clean_mfg = if is_oem_placeholder(&manufacturer) {
        read_reg_string(HKEY_LOCAL_MACHINE, subkey, "BaseBoardManufacturer")
            .unwrap_or_else(|| "ASUS".to_string())
    } else {
        manufacturer
    };

    if is_oem_placeholder(&clean_mfg) {
        clean_mfg = "ASUS".to_string();
    }

    if is_oem_placeholder(&model) {
        model = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "BaseBoardProduct")
            .unwrap_or_else(|| "ROG Strix PC".to_string());
    }

    if is_oem_placeholder(&model) {
        model = "Showroom PC".to_string();
    }

    let brand = clean_mfg
        .replace("ASUSTeK COMPUTER INC.", "ASUS")
        .replace("ASUSTeK Computer Inc.", "ASUS")
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

pub fn parse_battery_wh_from_xml(xml: &str) -> Option<u32> {
    let mut total_mwh: u64 = 0;

    // First scan <Batteries> block if it exists
    let batteries_block = if let Some(start) = xml.find("<Batteries>") {
        if let Some(end) = xml[start..].find("</Batteries>") {
            &xml[start..start + end]
        } else {
            xml
        }
    } else {
        xml
    };

    let mut cursor = batteries_block;
    while let Some(pos) = cursor.find("<DesignCapacity>") {
        let after = &cursor[pos + 16..];
        if let Some(end_pos) = after.find("</DesignCapacity>") {
            let num_str = after[..end_pos].trim();
            if let Ok(mwh) = num_str.parse::<u64>() {
                total_mwh += mwh;
            }
            cursor = &after[end_pos + 17..];
        } else {
            break;
        }
    }

    if total_mwh > 0 {
        return Some(mwh_to_wh(total_mwh));
    }

    // Fallback: check <RuntimeEstimates><DesignCapacity><Capacity>...
    if let Some(pos) = xml.find("<DesignCapacity") {
        let slice = &xml[pos..];
        if let Some(end_pos) = slice.find("</DesignCapacity>") {
            let inner = &slice[..end_pos];
            if let Some(cap_pos) = inner.find("<Capacity>") {
                let cap_slice = &inner[cap_pos + 10..];
                if let Some(cap_end) = cap_slice.find("</Capacity>") {
                    let num_str = cap_slice[..cap_end].trim();
                    if let Ok(mwh) = num_str.parse::<u64>() {
                        if mwh > 0 {
                            return Some(mwh_to_wh(mwh));
                        }
                    }
                }
            }
        }
    }

    // Fallback: check <FullChargeCapacity>
    if let Some(pos) = xml.find("<FullChargeCapacity>") {
        let after = &xml[pos + 20..];
        if let Some(end_pos) = after.find("</FullChargeCapacity>") {
            let num_str = after[..end_pos].trim();
            if let Ok(mwh) = num_str.parse::<u64>() {
                if mwh > 0 {
                    return Some(mwh_to_wh(mwh));
                }
            }
        }
    }

    None
}

fn mwh_to_wh(val: u64) -> u32 {
    if val >= 200 {
        ((val as f64) / 1000.0).round() as u32
    } else {
        val as u32
    }
}

pub fn detect_battery_capacity_wh() -> Option<u32> {
    let temp_dir = std::env::temp_dir();
    let report_path = temp_dir.join(format!("zenit_bat_{}.xml", std::process::id()));
    let report_str = report_path.to_string_lossy().to_string();

    let mut cmd = std::process::Command::new("powercfg");
    cmd.args(["/batteryreport", "/xml", "/output", &report_str]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    if let Ok(res) = cmd.output() {
        if res.status.success() && report_path.exists() {
            if let Ok(xml) = std::fs::read_to_string(&report_path) {
                let _ = std::fs::remove_file(&report_path);
                return parse_battery_wh_from_xml(&xml);
            }
            let _ = std::fs::remove_file(&report_path);
        }
    }
    None
}

pub fn get_battery_info() -> (String, u8, bool) {
    // 1. Detect physical battery capacity in Wh via powercfg /batteryreport
    if let Some(wh) = detect_battery_capacity_wh() {
        let mut percent = 100;
        let mut is_charging = true;
        unsafe {
            let mut power_status: SYSTEM_POWER_STATUS = std::mem::zeroed();
            if GetSystemPowerStatus(&mut power_status) != 0 {
                percent = if power_status.BatteryLifePercent <= 100 {
                    power_status.BatteryLifePercent
                } else {
                    100
                };
                is_charging = power_status.ACLineStatus == 1;
            }
        }
        return (format!("Batería {} Wh", wh), percent, is_charging);
    }

    // 2. Fallback to Win32 power status API
    unsafe {
        let mut power_status: SYSTEM_POWER_STATUS = std::mem::zeroed();
        if GetSystemPowerStatus(&mut power_status) != 0 {
            // BatteryFlag 128 indicates no system battery installed (Desktop PC)
            if power_status.BatteryFlag & 128 != 0 {
                return ("Hasta 8 horas".to_string(), 100, true);
            }

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
                    format!("Hasta {} horas", hours)
                } else {
                    format!("{}% (~{}m)", percent, mins)
                }
            } else if is_charging {
                "Hasta 8 horas".to_string()
            } else {
                format!("{}%", percent)
            };

            return (status_str, percent, is_charging);
        }
    }
    ("Hasta 8 horas".to_string(), 100, true)
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

    #[test]
    fn test_brand_and_model_cleanup() {
        let specs = detect_hardware();
        assert!(!specs.brand.to_lowercase().contains("to be filled"), "Brand must not contain raw OEM placeholder");
        assert!(!specs.model.to_lowercase().contains("to be filled"), "Model must not contain raw OEM placeholder");
        assert!(!specs.brand.is_empty(), "Brand should not be empty");
        assert!(!specs.model.is_empty(), "Model should not be empty");
    }

    #[test]
    fn test_parse_battery_wh_from_xml_standard() {
        let xml = r#"<BatteryReport>
            <Batteries>
                <Battery>
                    <DesignCapacity>42000</DesignCapacity>
                    <FullChargeCapacity>41200</FullChargeCapacity>
                </Battery>
            </Batteries>
        </BatteryReport>"#;
        assert_eq!(parse_battery_wh_from_xml(xml), Some(42));
    }

    #[test]
    fn test_parse_battery_wh_from_xml_nested_capacity() {
        let xml = r#"<BatteryReport>
            <RuntimeEstimates>
                <DesignCapacity>
                    <Capacity>56000</Capacity>
                </DesignCapacity>
            </RuntimeEstimates>
        </BatteryReport>"#;
        assert_eq!(parse_battery_wh_from_xml(xml), Some(56));
    }

    #[test]
    fn test_parse_battery_wh_from_xml_empty_desktop() {
        let xml = r#"<BatteryReport>
            <Batteries></Batteries>
            <RuntimeEstimates>
                <DesignCapacity><Capacity>0</Capacity></DesignCapacity>
            </RuntimeEstimates>
        </BatteryReport>"#;
        assert_eq!(parse_battery_wh_from_xml(xml), None);
    }
}

