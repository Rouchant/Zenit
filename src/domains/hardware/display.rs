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

/// Sets screen brightness to 100% on laptops and supported monitors (WMI).
/// Runs on a background thread without opening any window or blocking UI.
pub fn set_max_brightness() {
    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(|| {
            use std::process::Command;
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let ps_script = r#"
                try {
                    $m = Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods -ErrorAction SilentlyContinue
                    if ($m) {
                        $m | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{ Timeout = 1; Brightness = 100 } | Out-Null
                    } else {
                        $w = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction SilentlyContinue
                        if ($w) { $w.WmiSetBrightness(1, 100) | Out-Null }
                    }
                } catch {}
            "#;
            let _ = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", ps_script])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        });
    }
}
