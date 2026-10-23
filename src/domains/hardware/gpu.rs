use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::OnceLock;
use windows_sys::Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW};

static NVIDIA_POWER_LIMIT: OnceLock<Option<String>> = OnceLock::new();

/// Asigna una puntuación a la GPU según su fabricante y tipo para priorizar dedicadas sobre integradas.
pub fn rate_gpu(name: &str) -> i32 {
    let name_up = name.to_uppercase();
    if name_up.contains("NVIDIA") || name_up.contains("RTX") || name_up.contains("GTX") {
        10
    } else if name_up.contains("RX ")
        || name_up.contains("RX6")
        || name_up.contains("RX7")
        || name_up.contains("RX5")
        || name_up.contains("RX4")
    {
        8
    } else if name_up.contains("ARC") {
        5
    } else if name_up.contains("UHD") || name_up.contains("RADEON") || name_up.contains("IRIS") {
        2
    } else {
        1
    }
}

/// Consulta nvidia-smi para obtener el límite máximo de potencia (Wattage / TGP) de la tarjeta.
pub fn get_nvidia_watts() -> Option<String> {
    NVIDIA_POWER_LIMIT
        .get_or_init(|| {
            use std::os::windows::process::CommandExt;
            use std::process::Command;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let script = r#"$val = (nvidia-smi -q -d POWER | Select-String "Max Power Limit" | Where-Object { $_ -notmatch "N/A" }); if ($val) { [int][float]($val.ToString().Split(':')[1].Replace('W','').Trim()) }"#;
            if let Ok(output) = Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", script])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !stdout.is_empty() && stdout.chars().all(|c| c.is_numeric()) {
                    return Some(stdout);
                }
            }
            None
        })
        .clone()
}

pub fn get_gpu_name() -> String {
    let mut best_gpu = "Gráficos Integrados".to_string();
    let mut best_score = 0;

    unsafe {
        let mut dev: DISPLAY_DEVICEW = std::mem::zeroed();
        dev.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

        let mut index = 0;
        while EnumDisplayDevicesW(std::ptr::null(), index, &mut dev, 0) != 0 {
            let name_slice = &dev.DeviceString;
            let end_idx = name_slice.iter().position(|&c| c == 0).unwrap_or(name_slice.len());
            let os_str = OsString::from_wide(&name_slice[..end_idx]);
            if let Ok(name) = os_str.into_string() {
                let cleaned = name
                    .replace("(R)", "")
                    .replace("(TM)", "")
                    .replace("  ", " ")
                    .trim()
                    .to_string();

                if !cleaned.is_empty() {
                    let score = rate_gpu(&cleaned);
                    if score > best_score {
                        best_score = score;
                        best_gpu = cleaned;
                    }
                }
            }
            index += 1;
            dev.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
        }
    }

    if best_score >= 10 {
        if let Some(watts) = get_nvidia_watts() {
            best_gpu = format!("{} {}W", best_gpu, watts);
        }
    }

    best_gpu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_gpu() {
        let dedicated_nvidia = rate_gpu("NVIDIA GeForce RTX 4060 Laptop GPU");
        let dedicated_amd = rate_gpu("AMD Radeon RX 7600S");
        let arc_intel = rate_gpu("Intel Arc A770M");
        let integrated_intel = rate_gpu("Intel UHD Graphics");
        let integrated_amd = rate_gpu("AMD Radeon Graphics");
        let generic = rate_gpu("Basic Display Adapter");

        assert!(dedicated_nvidia > dedicated_amd, "NVIDIA debería tener mayor prioridad");
        assert!(dedicated_amd > arc_intel, "RX debería ganarle a Arc");
        assert!(arc_intel > integrated_intel, "Arc debería ganarle a UHD");
        assert_eq!(dedicated_nvidia, 10);
        assert_eq!(dedicated_amd, 8);
        assert_eq!(arc_intel, 5);
        assert_eq!(integrated_intel, 2);
        assert_eq!(integrated_amd, 2);
        assert_eq!(generic, 1);
    }
}
