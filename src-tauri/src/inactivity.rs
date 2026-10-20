use windows_sys::Win32::System::SystemInformation::GetTickCount;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

pub fn get_system_idle_seconds() -> u64 {
    unsafe {
        let mut lii: LASTINPUTINFO = std::mem::zeroed();
        lii.cbSize = std::mem::size_of::<LASTINPUTINFO>() as u32;

        if GetLastInputInfo(&mut lii) != 0 {
            let current_tick = GetTickCount();
            let idle_ms = current_tick.saturating_sub(lii.dwTime);
            return (idle_ms / 1000) as u64;
        }
    }
    0
}
