use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS,
};

/// Aproxima la tasa de refresco reportada por Windows (ej: 59Hz, 143Hz, 164Hz)
/// al valor comercial estándar más cercano si la diferencia es de 10Hz o menor.
pub fn approximate_refresh_rate(hz: u32) -> u32 {
    let commercial_rates = [60, 75, 90, 100, 120, 144, 165, 240, 360, 480, 540];
    let mut closest = hz;
    let mut min_diff = i64::MAX;
    for &r in &commercial_rates {
        let diff = (hz as i64 - r as i64).abs();
        if diff < min_diff {
            min_diff = diff;
            closest = r;
        }
    }
    if min_diff <= 10 {
        closest
    } else {
        hz
    }
}

/// Obtiene la resolución nativa de la pantalla primaria (ancho, alto).
pub fn get_display_raw_resolution() -> (u32, u32) {
    unsafe {
        let mut devmode: DEVMODEW = std::mem::zeroed();
        devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        if EnumDisplaySettingsW(std::ptr::null(), ENUM_CURRENT_SETTINGS, &mut devmode) != 0 {
            return (devmode.dmPelsWidth, devmode.dmPelsHeight);
        }
    }
    (1920, 1080)
}

/// Determina si la relación de aspecto de la pantalla es cercana a 16:10 (>= 1.55 y <= 1.68)
pub fn is_display_16_10() -> bool {
    let (w, h) = get_display_raw_resolution();
    if h == 0 {
        return false;
    }
    let ratio = w as f32 / h as f32;
    // 16:10 = 1.6, 16:9 = 1.777
    (ratio - 1.6).abs() < (ratio - 1.7777).abs()
}

pub fn get_display_and_hz() -> (String, u32) {
    unsafe {
        let mut devmode: DEVMODEW = std::mem::zeroed();
        devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;

        if EnumDisplaySettingsW(std::ptr::null(), ENUM_CURRENT_SETTINGS, &mut devmode) != 0 {
            let width = devmode.dmPelsWidth;
            let height = devmode.dmPelsHeight;
            let raw_hz = devmode.dmDisplayFrequency;
            let hz = approximate_refresh_rate(raw_hz);

            let tag = match (width, height) {
                (1920, 1080) => " (Full HD)",
                (1920, 1200) => " (WUXGA)",
                (2560, 1440) => " (2K QHD)",
                (2560, 1600) => " (2.5K)",
                (2880, 1800) => " (2.8K OLED)",
                (3000, 2000) => " (3K)",
                (3200, 2000) => " (3.2K)",
                (3840, 2160) => " (4K UHD)",
                (3840, 2400) => " (UHD+)",
                (1366, 768) => " (HD)",
                _ => "",
            };

            return (format!("{} x {}{} @ {}Hz", width, height, tag, hz), hz);
        }
    }
    ("1920 x 1080 (Full HD) @ 60Hz".to_string(), 60)
}

static LAST_BRIGHTNESS_SET: AtomicU64 = AtomicU64::new(0);

/// Fuerza el brillo al 100% y bloquea la suspensión de pantalla/sistema de Windows.
/// Se ejecuta de forma asíncrona sin ventanas emergentes.
pub fn set_max_brightness() {
    #[cfg(target_os = "windows")]
    {
        // 1. Bloqueo Nativo (Hard Block): Informa a Windows que la pantalla y el sistema
        // deben mantenerse activos, ignorando planes de suspensión o ahorro.
        unsafe {
            use windows_sys::Win32::System::Power::{
                SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
            };
            SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
        }

        // 2. Ajuste de Brillo (Rate-limited a 10s para evitar sobrecoste de procesos)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let last = LAST_BRIGHTNESS_SET.load(Ordering::Relaxed);
        if now.saturating_sub(last) < 10 {
            return;
        }
        LAST_BRIGHTNESS_SET.store(now, Ordering::Relaxed);

        std::thread::spawn(|| {
            use std::os::windows::process::CommandExt;
            use std::process::Command;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let cmd = "try { $m = Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods -ErrorAction SilentlyContinue; if ($m) { $m | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{ Timeout = 1; Brightness = 100 } | Out-Null } else { $w = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction SilentlyContinue; if ($w) { $w.WmiSetBrightness(1, 100) | Out-Null } } } catch {}";
            let _ = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", cmd])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approximate_refresh_rate() {
        assert_eq!(approximate_refresh_rate(59), 60);
        assert_eq!(approximate_refresh_rate(60), 60);
        assert_eq!(approximate_refresh_rate(74), 75);
        assert_eq!(approximate_refresh_rate(143), 144);
        assert_eq!(approximate_refresh_rate(164), 165);
        assert_eq!(approximate_refresh_rate(165), 165);
        assert_eq!(approximate_refresh_rate(239), 240);
        assert_eq!(approximate_refresh_rate(360), 360);
        assert_eq!(approximate_refresh_rate(0), 0);
        assert_eq!(approximate_refresh_rate(13), 13);
    }

    #[test]
    fn test_display_raw_resolution_non_zero() {
        let (w, h) = get_display_raw_resolution();
        assert!(w > 0);
        assert!(h > 0);
    }
}
