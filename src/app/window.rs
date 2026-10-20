use super::state::{MAIN_HWND, FLOATING_HWND};
use crate::MainWindow;
use slint::ComponentHandle;
use std::sync::atomic::Ordering;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{HWND, LPARAM};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumThreadWindows, GetParent, GetSystemMetrics, IsWindow, SetForegroundWindow, SetWindowPos,
    ShowWindow, HWND_TOP, HWND_TOPMOST, SM_CXSCREEN, SM_CYSCREEN, SWP_SHOWWINDOW, SW_HIDE,
    SW_MINIMIZE, SW_RESTORE, SW_SHOW,
};

pub fn init_main_window(window: &MainWindow) {
    window.show().expect("Failed to show Slint window");
    window.window().set_fullscreen(true);

    #[cfg(target_os = "windows")]
    unsafe {
        unsafe extern "system" fn enum_main_win(hwnd: HWND, _: LPARAM) -> i32 {
            if unsafe { GetParent(hwnd) }.is_null() {
                MAIN_HWND.store(hwnd as isize, Ordering::SeqCst);
                return 0; // Found main showroom window
            }
            1
        }

        let thread_id = GetCurrentThreadId();
        EnumThreadWindows(thread_id, Some(enum_main_win), 0);

        let hwnd = MAIN_HWND.load(Ordering::SeqCst) as HWND;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        println!("[Showroom] Resolucion detectada: {}x{}, HWND: {:?}", screen_w, screen_h, hwnd);
        window.set_screen_width(screen_w as f32);
        window.set_screen_height(screen_h as f32);

        if !hwnd.is_null() {
            SetWindowPos(hwnd, HWND_TOP, 0, 0, screen_w, screen_h, SWP_SHOWWINDOW);
        }
    }
}

pub fn minimize_main_window() {
    #[cfg(target_os = "windows")]
    unsafe {
        let main_hwnd = MAIN_HWND.load(Ordering::SeqCst) as HWND;
        if !main_hwnd.is_null() {
            ShowWindow(main_hwnd, SW_MINIMIZE);
            ShowWindow(main_hwnd, SW_HIDE);
        }
    }
}

pub fn restore_main_window() {
    #[cfg(target_os = "windows")]
    unsafe {
        let main_hwnd = MAIN_HWND.load(Ordering::SeqCst) as HWND;
        if !main_hwnd.is_null() {
            ShowWindow(main_hwnd, SW_SHOW);
            ShowWindow(main_hwnd, SW_RESTORE);
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            SetWindowPos(main_hwnd, HWND_TOP, 0, 0, screen_w, screen_h, SWP_SHOWWINDOW);
            SetForegroundWindow(main_hwnd);
        }
    }
}

pub fn position_floating_button() {
    #[cfg(target_os = "windows")]
    unsafe {
        let main_hwnd = MAIN_HWND.load(Ordering::SeqCst) as HWND;
        let mut floating_hwnd = FLOATING_HWND.load(Ordering::SeqCst) as HWND;

        if floating_hwnd.is_null() || IsWindow(floating_hwnd) == 0 {
            unsafe extern "system" fn enum_btn(hwnd: HWND, lparam: LPARAM) -> i32 {
                let main_hwnd = lparam as HWND;
                if hwnd != main_hwnd && unsafe { GetParent(hwnd) }.is_null() {
                    FLOATING_HWND.store(hwnd as isize, Ordering::SeqCst);
                    return 0; // Found floating button window
                }
                1
            }

            let thread_id = GetCurrentThreadId();
            EnumThreadWindows(thread_id, Some(enum_btn), main_hwnd as LPARAM);
            floating_hwnd = FLOATING_HWND.load(Ordering::SeqCst) as HWND;
        }

        if !floating_hwnd.is_null() {
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let w = 220;
            let h = 60;
            let x = screen_w - w - 40;
            let y = screen_h - h - 60;
            SetWindowPos(floating_hwnd, HWND_TOPMOST, x, y, w, h, SWP_SHOWWINDOW);
            SetForegroundWindow(floating_hwnd);
        }
    }
}
