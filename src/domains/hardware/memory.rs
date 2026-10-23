use std::sync::OnceLock;
use sysinfo::System;

static CACHED_RAM_TYPE: OnceLock<String> = OnceLock::new();

/// Formatea la capacidad de RAM, redondeando para absorber la memoria reservada por hardware/iGPU.
pub fn get_ram_display(total_bytes: u64) -> String {
    let gb = total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    let size = (gb / 2.0).round() * 2.0;
    format!("{:.0}GB", if size == 0.0 { gb.round() } else { size })
}

/// Detecta la tecnología de RAM (DDR4, DDR5, LPDDR5X) y su velocidad en MT/s vía SMBIOS
/// con fallback heurístico según la plataforma del procesador.
pub fn detect_ram_type(cpu_name: &str) -> String {
    CACHED_RAM_TYPE
        .get_or_init(|| {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                use std::process::Command;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let script = r#"$m = Get-CimInstance Win32_PhysicalMemory -ErrorAction SilentlyContinue | Select-Object -First 1 SMBIOSMemoryType, ConfiguredClockSpeed; if ($m) { "$($m.SMBIOSMemoryType)|$($m.ConfiguredClockSpeed)" }"#;
                if let Ok(output) = Command::new("powershell")
                    .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", script])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output()
                {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if let Some((smbios_str, speed_str)) = stdout.split_once('|') {
                        let smbios = smbios_str.trim().parse::<u64>().unwrap_or(0);
                        let speed = speed_str.trim().parse::<u64>().unwrap_or(0);
                        let mut r_type = match smbios {
                            20 => "DDR",
                            21 | 22 => "DDR2",
                            24 => "DDR3",
                            26 => "DDR4",
                            29 => "LPDDR3",
                            30 | 31 => "LPDDR4",
                            34 => "DDR5",
                            35 => "LPDDR5",
                            _ => "",
                        };
                        if r_type == "LPDDR5" && speed >= 6000 {
                            r_type = "LPDDR5X";
                        }
                        if !r_type.is_empty() {
                            if speed > 0 {
                                return format!("{} - {} MT/s", r_type, speed);
                            }
                            return r_type.to_string();
                        }
                    }
                }
            }

            // Fallback heurístico por arquitectura de procesador
            let lower = cpu_name.to_lowercase();
            if lower.contains("ultra")
                || lower.contains("ryzen ai")
                || lower.contains("snapdragon")
                || lower.contains("8000")
                || lower.contains("7000")
                || lower.contains("serie 1")
            {
                "DDR5".to_string()
            } else {
                "DDR4".to_string()
            }
        })
        .clone()
}

pub fn get_ram_info(sys: &System, cpu_name: &str) -> (String, String) {
    let ram = get_ram_display(sys.total_memory());
    let ram_type = detect_ram_type(cpu_name);
    (ram, ram_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ram_display_rounding() {
        // Absorción de memoria reservada por gráficos integrados (GiB)
        // 16 GiB con 512 MB reservados por iGPU
        assert_eq!(get_ram_display(16 * 1024 * 1024 * 1024 - 512 * 1024 * 1024), "16GB");
        // 8 GiB con 512 MB reservados por iGPU
        assert_eq!(get_ram_display(8 * 1024 * 1024 * 1024 - 512 * 1024 * 1024), "8GB");
        // 32 GiB con 1024 MB reservados por iGPU
        assert_eq!(get_ram_display(32 * 1024 * 1024 * 1024 - 1024 * 1024 * 1024), "32GB");
    }
}
