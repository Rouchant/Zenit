use crate::domains::hardware::registry::read_reg_string;
use sysinfo::System;
use windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE;

pub fn is_oem_placeholder(s: &str) -> bool {
    let l = s.to_lowercase();
    l.contains("to be filled")
        || l.contains("system manufacturer")
        || l.contains("system product name")
        || l.contains("o.e.m.")
        || l.contains("default string")
        || l.trim().is_empty()
}

pub fn get_laptop_brand_and_model() -> (String, String) {
    let subkey = r"HARDWARE\DESCRIPTION\System\BIOS";
    let manufacturer = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "SystemManufacturer")
        .unwrap_or_else(|| "ASUS".to_string());
    let mut model = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "SystemProductName")
        .unwrap_or_else(|| "".to_string());

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

pub fn get_os_info() -> String {
    let os_name = System::name().unwrap_or_else(|| "Windows".to_string());
    let os_version = System::os_version().unwrap_or_default();
    if os_name.contains("Windows") {
        if os_version.starts_with("11")
            || os_version.contains("22")
            || os_version.contains("23")
            || os_version.contains("24")
        {
            "Windows 11 Home".to_string()
        } else {
            "Windows 10 Home".to_string()
        }
    } else {
        format!("{} {}", os_name, os_version)
    }
}
