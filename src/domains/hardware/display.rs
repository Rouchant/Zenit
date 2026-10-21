use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS,
};

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

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static LAST_BRIGHTNESS_SET: AtomicU64 = AtomicU64::new(0);

/// Sets screen brightness to 100% on laptops and supported monitors (WMI).
/// Runs on a background thread without opening any window or blocking UI.
/// Rate-limited to at most once every 10 seconds to avoid process overhead.
pub fn set_max_brightness() {
    #[cfg(target_os = "windows")]
    {
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
            use std::process::Command;
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let cmd = "try { $m = Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods -ErrorAction SilentlyContinue; if ($m) { $m | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{ Timeout = 1; Brightness = 100 } | Out-Null } else { $w = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction SilentlyContinue; if ($w) { $w.WmiSetBrightness(1, 100) | Out-Null } } } catch {}";
            let _ = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", cmd])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        });
    }
}
