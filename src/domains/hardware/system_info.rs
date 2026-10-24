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

/// Normaliza nombres técnicos de fabricantes a sus nombres comerciales conocidos.
pub fn clean_brand_name(raw: &str) -> String {
    let r = raw.to_uppercase();
    if r.contains("ASUSTEK") || r.contains("ASUS") {
        "ASUS".to_string()
    } else if r.contains("HEWLETT-PACKARD") || r.split_whitespace().any(|w| w == "HP") || r == "HP" {
        "HP".to_string()
    } else if r.contains("SAMSUNG") {
        "Samsung".to_string()
    } else if r.contains("DELL") {
        "Dell".to_string()
    } else if r.contains("LENOVO") {
        "Lenovo".to_string()
    } else if r.contains("ACER") {
        "Acer".to_string()
    } else if r.contains("ASROCK") {
        "ASRock".to_string()
    } else if r.contains("GIGABYTE") {
        "Gigabyte".to_string()
    } else if r.contains("MSI") || r.contains("MICRO-STAR") {
        "MSI".to_string()
    } else if r.contains("VIRTUALBOX") {
        "VirtualBox".to_string()
    } else {
        raw.trim().to_string()
    }
}

/// Refina el nombre del modelo eliminando ruido repetitivo, normalizando espaciados de ASUS y deduplicando palabras.
/// La marca NO se antepone al modelo resultante para evitar redundancia visual con el logo.
pub fn refine_model_name(brand: &str, model: &str) -> String {
    let noise = [
        "ASUSTEK", "COMPUTER", "INC", "CORP", "CORPORATION", "LTD", "SYSTEMS", "PRODUCT", "NAME",
        "LAPTOP",
    ];

    let mut clean = model
        .replace('_', " ")
        .replace("ASUSLaptop", " Laptop ")
        .replace('-', " ")
        .trim()
        .to_string();

    while clean.contains("  ") {
        clean = clean.replace("  ", " ");
    }

    let words: Vec<&str> = clean.split_whitespace().collect();
    let mut unique_words: Vec<String> = Vec::new();
    let brand_up = brand.to_uppercase();

    for &word in &words {
        let word_up = word.to_uppercase().replace('.', "");
        if noise.contains(&word_up.as_str()) {
            continue;
        }
        if word_up == brand_up || (brand_up.len() > 2 && word_up.contains(&brand_up)) {
            continue;
        }

        if let Some(last) = unique_words.last() {
            let last_up = last.to_uppercase();
            if word_up == last_up || word_up.starts_with(&last_up) || last_up.starts_with(&word_up) {
                if word.len() > last.len() {
                    unique_words.pop();
                    unique_words.push(word.to_string());
                }
                continue;
            }
        }
        unique_words.push(word.to_string());
    }

    let result_model = unique_words.join(" ");
    if result_model.is_empty() {
        brand.to_string()
    } else {
        result_model
    }
}

pub fn get_laptop_brand_and_model() -> (String, String) {
    let subkey = r"HARDWARE\DESCRIPTION\System\BIOS";
    let manufacturer = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "SystemManufacturer")
        .unwrap_or_else(|| "ASUS".to_string());
    let mut model = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "SystemProductName")
        .unwrap_or_else(|| "".to_string());

    // Fallback a Placa Base si la información inicial es genérica (O.E.M, Default string, etc.)
    let raw_brand = if is_oem_placeholder(&manufacturer) {
        read_reg_string(HKEY_LOCAL_MACHINE, subkey, "BaseBoardManufacturer")
            .unwrap_or_else(|| "ASUS".to_string())
    } else {
        manufacturer
    };

    let brand = clean_brand_name(&raw_brand);

    if is_oem_placeholder(&model) {
        model = read_reg_string(HKEY_LOCAL_MACHINE, subkey, "BaseBoardProduct")
            .unwrap_or_else(|| "ROG Strix PC".to_string());
    }

    if is_oem_placeholder(&model) {
        model = "Showroom PC".to_string();
    }

    let final_model = if brand.to_uppercase().contains("VIRTUALBOX") {
        "Virtual Machine".to_string()
    } else {
        refine_model_name(&brand, &model)
    };

    (brand, final_model)
}

pub fn detect_os_badge(os_str: &str) -> Option<String> {
    let low = os_str.to_lowercase();
    if low.contains("ltsc") {
        return Some("LTSC".to_string());
    } else if low.contains("pro") || low.contains("professional") {
        return Some("Pro".to_string());
    } else if low.contains("home") || low.contains("core") {
        return Some("Home".to_string());
    } else if low.contains("enterprise") {
        return Some("Enterprise".to_string());
    } else if low.contains("education") {
        return Some("Education".to_string());
    } else if low.contains("workstation") {
        return Some("Workstation".to_string());
    } else if low.contains("server") {
        return Some("Server".to_string());
    }

    #[cfg(target_os = "windows")]
    if low.contains("windows") || low.contains("win") || low.trim().is_empty() {
        use windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE;
        if let Some(ed) = read_reg_string(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "EditionID",
        ) {
            let ed_low = ed.to_lowercase();
            if ed_low.contains("ltsc") || (ed_low.contains("enterprise") && ed_low.contains('s')) {
                return Some("LTSC".to_string());
            } else if ed_low.contains("pro") {
                return Some("Pro".to_string());
            } else if ed_low.contains("home") || ed_low.contains("core") {
                return Some("Home".to_string());
            } else if ed_low.contains("enterprise") {
                return Some("Enterprise".to_string());
            } else if ed_low.contains("education") {
                return Some("Education".to_string());
            }
        }
    }

    None
}

pub fn get_os_info() -> String {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE;
        let reg_sub = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";
        let prod_name = read_reg_string(HKEY_LOCAL_MACHINE, reg_sub, "ProductName");
        let build_num: u32 = read_reg_string(HKEY_LOCAL_MACHINE, reg_sub, "CurrentBuild")
            .and_then(|b| b.parse().ok())
            .unwrap_or(0);

        if let Some(mut name) = prod_name {
            if !name.is_empty() {
                // Windows 11 returns "Windows 10 ..." in ProductName for builds >= 22000
                if build_num >= 22000 && name.starts_with("Windows 10") {
                    name = name.replacen("Windows 10", "Windows 11", 1);
                }
                return name;
            }
        }

        if let Some(ed) = read_reg_string(HKEY_LOCAL_MACHINE, reg_sub, "EditionID") {
            let win_ver = if build_num >= 22000 { "11" } else { "10" };
            let ed_clean = match ed.to_lowercase().as_str() {
                "professional" | "pro" => "Pro",
                "core" | "home" => "Home",
                "enterprises" | "enterprise" => "Enterprise",
                "enterprisesn" => "Enterprise LTSC",
                "education" => "Education",
                _ => ed.as_str(),
            };
            return format!("Windows {} {}", win_ver, ed_clean);
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_brand_name() {
        assert_eq!(clean_brand_name("ASUSTeK COMPUTER INC."), "ASUS");
        assert_eq!(clean_brand_name("Hewlett-Packard"), "HP");
        assert_eq!(clean_brand_name("Micro-Star International Co., Ltd."), "MSI");
        assert_eq!(clean_brand_name("LENOVO"), "Lenovo");
        assert_eq!(clean_brand_name("Dell Inc."), "Dell");
    }

    #[test]
    fn test_refine_model_name() {
        assert_eq!(refine_model_name("ASUS", "ASUS TUF Gaming F15"), "TUF Gaming F15");
        assert_eq!(refine_model_name("HP", "HP Laptop 15s-eq2xxx"), "15s eq2xxx");
        assert_eq!(refine_model_name("Lenovo", "82K2"), "82K2");
        assert_eq!(refine_model_name("Lenovo", "Lenovo LOQ 15AHP10"), "LOQ 15AHP10");
        assert_eq!(refine_model_name("ASUS", "ASUS Vivobook 15 E1504GAB"), "Vivobook 15 E1504GAB");
    }

    #[test]
    fn test_detect_os_badge() {
        assert_eq!(detect_os_badge("Windows 11 Home"), Some("Home".to_string()));
        assert_eq!(detect_os_badge("Windows 11 Pro"), Some("Pro".to_string()));
        assert_eq!(detect_os_badge("Windows 10 Enterprise LTSC 2021"), Some("LTSC".to_string()));
        assert_eq!(detect_os_badge("Windows 10 Education"), Some("Education".to_string()));
        assert_eq!(detect_os_badge("Linux Ubuntu"), None);
    }
}
