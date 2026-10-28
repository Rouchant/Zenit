use super::state::{MAIN_HWND, FLOATING_HWND};
use crate::MainWindow;
use slint::ComponentHandle;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, EnumThreadWindows, GetParent, GetSystemMetrics, IsWindow, SetForegroundWindow,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, GWLP_WNDPROC, HWND_TOP, HWND_TOPMOST, SC_CLOSE,
    SM_CXSCREEN, SM_CYSCREEN, SWP_SHOWWINDOW, SW_HIDE, SW_RESTORE, SW_SHOW,
    WM_CLOSE, WM_CONTEXTMENU, WM_INITMENUPOPUP, WM_SYSCOMMAND, WNDPROC,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};

static OLD_MAIN_WNDPROC: AtomicIsize = AtomicIsize::new(0);
pub static ALLOW_CLOSE: AtomicBool = AtomicBool::new(false);

const WM_POWERBROADCAST: u32 = 0x0218;
const PBT_APMRESUMEAUTOMATIC: u32 = 0x0012;
const PBT_APMRESUMESUSPEND: u32 = 0x0007;
const SC_KEYMENU: u32 = 0xF100;

/// Configura la aplicación como DPI-Aware Per-Monitor v2 para que Windows no escale artificialmente
/// ni deforme los elementos visuales sin importar si el usuario tiene 125%, 150% o resoluciones 16:10.
pub fn enable_dpi_awareness_v2() {
    #[cfg(target_os = "windows")]
    unsafe {
        let res = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        if res != 0 {
            println!("[DPI] Windows Per-Monitor DPI Awareness v2 activado con éxito.");
        }
    }
}

/// Permite el cierre de la ventana cuando se solicita conscientemente (ej: salir de emergencia).
pub fn allow_app_close() {
    ALLOW_CLOSE.store(true, Ordering::SeqCst);
}

#[cfg(target_os = "windows")]
pub fn protect_main_window_proc(hwnd: HWND) {
    if OLD_MAIN_WNDPROC.load(Ordering::SeqCst) != 0 {
        return;
    }
    unsafe {
        let old = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, main_wndproc_subclass as *const () as isize);
        OLD_MAIN_WNDPROC.store(old, Ordering::SeqCst);
        println!("[Guardian] Subclase de ventana instalada para proteger WM_CLOSE y eventos de energía.");
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn main_wndproc_subclass(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let old_proc = OLD_MAIN_WNDPROC.load(Ordering::Relaxed);

    if ALLOW_CLOSE.load(Ordering::Relaxed) {
        if old_proc != 0 {
            unsafe {
                let old_proc_fn: WNDPROC = std::mem::transmute(old_proc);
                return CallWindowProcW(old_proc_fn, hwnd, msg, wparam, lparam);
            }
        }
        return 0;
    }

    if msg == WM_POWERBROADCAST {
        let event_type = wparam as u32;
        if event_type == PBT_APMRESUMEAUTOMATIC || event_type == PBT_APMRESUMESUSPEND {
            println!("[Power] Sistema despertando de suspensión. Re-activando pantalla y brillo...");
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
            }
            crate::domains::hardware::display::set_max_brightness();
        }
    }

    if msg == WM_CLOSE {
        println!("[Guardian] Interceptado WM_CLOSE en ventana de Slint. Cierre bloqueado para proteger el modo kiosco.");
        return 0;
    }

    if msg == WM_SYSCOMMAND {
        let cmd = (wparam & 0xFFF0) as u32;
        if cmd == SC_CLOSE || cmd == SC_KEYMENU {
            return 0;
        }
    }

    if msg == WM_INITMENUPOPUP || msg == WM_CONTEXTMENU {
        return 0;
    }

    if old_proc != 0 {
        unsafe {
            let old_proc_fn: WNDPROC = std::mem::transmute(old_proc);
            return CallWindowProcW(old_proc_fn, hwnd, msg, wparam, lparam);
        }
    }
    0
}

#[cfg(target_os = "windows")]
#[link(name = "winmm")]
unsafe extern "system" {
    fn timeBeginPeriod(uPeriod: u32) -> u32;
}

/// Desactiva el modo EcoQoS / Power Throttling en Windows 11 para evitar caídas de FPS
/// cuando la ventana pierde el foco o corre en segundo plano.
pub fn disable_power_throttling() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, ProcessPowerThrottling, SetProcessInformation,
            PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION, PROCESS_POWER_THROTTLING_STATE,
        };

        let mut state = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED | PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION,
            StateMask: 0,
        };

        let res = SetProcessInformation(
            GetCurrentProcess(),
            ProcessPowerThrottling,
            &mut state as *mut _ as *const _,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
        if res != 0 {
            println!("[Power] Windows 11 Power Throttling (EcoQoS) desactivado exitosamente.");
        }
    }
}


pub fn init_main_window(window: &MainWindow) {
    window.show().expect("Failed to show Slint window");
    window.window().set_fullscreen(true);

    #[cfg(target_os = "windows")]
    unsafe {
        // Set high precision 1ms timer resolution for buttery-smooth video and animations
        timeBeginPeriod(1);

        // Prevent Windows from sleeping or turning off the display
        SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);

        // Desactivar EcoQoS / Power Throttling en Windows 11
        disable_power_throttling();

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
            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_RESTORE);
            SetWindowPos(hwnd, HWND_TOP, 0, 0, screen_w, screen_h, SWP_SHOWWINDOW);
            SetForegroundWindow(hwnd);
            protect_main_window_proc(hwnd);
        }
    }
}

pub fn minimize_main_window() {
    #[cfg(target_os = "windows")]
    unsafe {
        let main_hwnd = MAIN_HWND.load(Ordering::SeqCst) as HWND;
        if !main_hwnd.is_null() {
            // Ocultamos directamente sin SW_MINIMIZE para evitar que Windows ejecute la animación
            // de colapso de pantalla completa que produce el pantallazo negro de 1 segundo
            ShowWindow(main_hwnd, SW_HIDE);
        }
    }
}

pub fn restore_main_window() {
    #[cfg(target_os = "windows")]
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
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
            let w = 160;
            let h = 120;
            let x = screen_w - w - 24;
            let y = (screen_h - h) / 2;
            ShowWindow(floating_hwnd, SW_RESTORE);
            SetWindowPos(floating_hwnd, HWND_TOPMOST, x, y, w, h, SWP_SHOWWINDOW);
            SetForegroundWindow(floating_hwnd);
        }
    }
    FB_WAS_FOREGROUND.store(true, Ordering::SeqCst);
}

static FB_WAS_FOREGROUND: AtomicBool = AtomicBool::new(true);

/// Devuelve `true` si el usuario tocó el icono de Zenit en la barra de tareas
/// mientras está en modo "Probar esta PC": la ventana flotante fue minimizada
/// (clic estando activa) o pasó a primer plano tras haber perdido el foco.
pub fn floating_button_taskbar_activated() -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, IsIconic};
        let fb = FLOATING_HWND.load(Ordering::SeqCst) as HWND;
        if fb.is_null() || IsWindow(fb) == 0 {
            return false;
        }
        if IsIconic(fb) != 0 {
            FB_WAS_FOREGROUND.store(true, Ordering::SeqCst);
            return true;
        }
        let is_fg = GetForegroundWindow() == fb;
        let was_fg = FB_WAS_FOREGROUND.swap(is_fg, Ordering::SeqCst);
        return is_fg && !was_fg;
    }
    #[allow(unreachable_code)]
    false
}
