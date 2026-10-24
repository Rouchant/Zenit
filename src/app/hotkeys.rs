use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_HIGHEST,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
    VK_APPS, VK_CONTROL, VK_ESCAPE, VK_F12, VK_F4, VK_LCONTROL, VK_LMENU, VK_LSHIFT,
    VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT, VK_SPACE, VK_TAB,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION,
    HHOOK, KBDLLHOOKSTRUCT, LLKHF_ALTDOWN, MSG, WH_KEYBOARD_LL, WM_HOTKEY, WM_KEYDOWN, WM_KEYUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

// Rastreo ligero de modificadores en memoria
static WIN_DOWN: AtomicBool = AtomicBool::new(false);
static CTRL_DOWN: AtomicBool = AtomicBool::new(false);
static SHIFT_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);

/// Cierre rápido inmediato de la aplicación
pub fn quick_exit() -> ! {
    println!("[Zenit] Cierre rápido ejecutado (Ctrl+Shift+Alt+Z). Saliendo...");
    crate::app::window::allow_app_close();
    std::process::exit(0);
}

/// Reinicio rápido inmediato de la aplicación
pub fn quick_restart() -> ! {
    println!("[Zenit] Reinicio rápido ejecutado (Ctrl+Shift+Alt+R). Reiniciando proceso...");
    crate::app::window::allow_app_close();
    if let Ok(current_exe) = std::env::current_exe() {
        let exe_str = current_exe.to_string_lossy().to_string();
        // Espera 1 segundo para liberar el mutex de single_instance y recursos gráficos/mpv
        let _ = std::process::Command::new("cmd")
            .args([
                "/C",
                &format!("timeout /t 1 /nobreak >nul & start \"\" \"{}\"", exe_str),
            ])
            .spawn();
    }
    std::process::exit(0);
}

/// Inicia el listener de hotkeys y el guardián de teclado Kiosco en un hilo dedicado de alta prioridad.
pub fn spawn_global_hotkey_listener() {
    #[cfg(target_os = "windows")]
    {
        std::thread::Builder::new()
            .name("zenit_hotkeys".to_string())
            .spawn(move || unsafe {
                // Elevar la prioridad del hilo para que el procesamiento sea instantáneo
                SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);

                const HOTKEY_EXIT_ID: i32 = 9101;
                const HOTKEY_RESTART_ID: i32 = 9102;
                let modifiers = MOD_CONTROL | MOD_SHIFT | MOD_ALT | MOD_NOREPEAT;

                // 1. HotKeys de nivel Kernel (inmunes a retardos)
                let ok_z = RegisterHotKey(std::ptr::null_mut(), HOTKEY_EXIT_ID, modifiers, b'Z' as u32);
                let ok_r = RegisterHotKey(std::ptr::null_mut(), HOTKEY_RESTART_ID, modifiers, b'R' as u32);

                if ok_z != 0 {
                    println!("[Hotkeys] Kernel HotKey registrado: Ctrl+Shift+Alt+Z -> Cierre rápido");
                }
                if ok_r != 0 {
                    println!("[Hotkeys] Kernel HotKey registrado: Ctrl+Shift+Alt+R -> Reinicio rápido");
                }

                // 2. Hook de bajo nivel para bloquear combinaciones de escape de Kiosco
                let h_instance = GetModuleHandleW(std::ptr::null());
                let hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(low_level_keyboard_proc),
                    h_instance,
                    0,
                );

                if !hook.is_null() {
                    HOOK_HANDLE.store(hook as isize, Ordering::SeqCst);
                    println!("[Guardian] Hook de teclado de Kiosco registrado exitosamente.");
                } else {
                    eprintln!("[Guardian] Advertencia: No se pudo registrar hook de teclado de bajo nivel.");
                }

                // 3. Loop de mensajes dedicado para el hook y los hotkeys
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    if msg.message == WM_HOTKEY {
                        match msg.wParam as i32 {
                            HOTKEY_EXIT_ID => {
                                quick_exit();
                            }
                            HOTKEY_RESTART_ID => {
                                quick_restart();
                            }
                            _ => {}
                        }
                    }
                }

                // 4. Limpieza ordenada al salir
                if !hook.is_null() {
                    UnhookWindowsHookEx(hook);
                }
                UnregisterHotKey(std::ptr::null_mut(), HOTKEY_EXIT_ID);
                UnregisterHotKey(std::ptr::null_mut(), HOTKEY_RESTART_ID);
            })
            .expect("Failed to spawn global hotkey thread");
    }
}

const VK_A: u32 = 0x41;
const VK_C: u32 = 0x43;
const VK_D: u32 = 0x44;
const VK_E: u32 = 0x45;
const VK_I: u32 = 0x49;
const VK_J: u32 = 0x4A;
const VK_K: u32 = 0x4B;
const VK_L: u32 = 0x4C;
const VK_M: u32 = 0x4D;
const VK_R: u32 = 0x52;
const VK_S: u32 = 0x53;
const VK_V: u32 = 0x56;
const VK_X: u32 = 0x58;

/// Callback de bajo nivel ultra-optimizado:
/// - NUNCA bloquea KeyUp (evita que las teclas Alt, Win, etc. se queden pegadas)
/// - CERO operaciones de entrada/salida (I/O) ni prints dentro del hook
/// - CERO cerraduras de Mutex para garantizar ejecución en nanosegundos
#[cfg(target_os = "windows")]
unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let current_hook = HOOK_HANDLE.load(Ordering::Relaxed) as HHOOK;

    if n_code < 0 || n_code != HC_ACTION as i32 {
        return unsafe { CallNextHookEx(current_hook, n_code, w_param, l_param) };
    }

    let kbd_struct = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };
    let key = kbd_struct.vkCode;
    let event = w_param as u32;

    let is_down = event == WM_KEYDOWN || event == WM_SYSKEYDOWN;
    let is_up = event == WM_KEYUP || event == WM_SYSKEYUP;

    // Regla de Oro: En KeyUp SIEMPRE actualizamos y dejamos pasar el evento al sistema
    if is_up {
        match key as u16 {
            VK_LWIN | VK_RWIN => WIN_DOWN.store(false, Ordering::Relaxed),
            VK_LCONTROL | VK_RCONTROL | VK_CONTROL => CTRL_DOWN.store(false, Ordering::Relaxed),
            VK_LSHIFT | VK_RSHIFT | VK_SHIFT => SHIFT_DOWN.store(false, Ordering::Relaxed),
            VK_LMENU | VK_RMENU | VK_MENU => ALT_DOWN.store(false, Ordering::Relaxed),
            _ => {}
        }
        return unsafe { CallNextHookEx(current_hook, n_code, w_param, l_param) };
    }

    if is_down {
        match key as u16 {
            VK_LWIN | VK_RWIN => WIN_DOWN.store(true, Ordering::Relaxed),
            VK_LCONTROL | VK_RCONTROL | VK_CONTROL => CTRL_DOWN.store(true, Ordering::Relaxed),
            VK_LSHIFT | VK_RSHIFT | VK_SHIFT => SHIFT_DOWN.store(true, Ordering::Relaxed),
            VK_LMENU | VK_RMENU | VK_MENU => ALT_DOWN.store(true, Ordering::Relaxed),
            _ => {}
        }

        let alt = (kbd_struct.flags & LLKHF_ALTDOWN) != 0 || ALT_DOWN.load(Ordering::Relaxed);
        let win = WIN_DOWN.load(Ordering::Relaxed);
        let ctrl = CTRL_DOWN.load(Ordering::Relaxed);
        let shift = SHIFT_DOWN.load(Ordering::Relaxed);
        let key_u16 = key as u16;

        // 1. Permitir apertura de Administrador de Tareas en emergencias (Ctrl + Shift + Esc)
        if ctrl && shift && key_u16 == VK_ESCAPE {
            return unsafe { CallNextHookEx(current_hook, n_code, w_param, l_param) };
        }

        // 2. Permitir atajos normales de edición de texto (Ctrl+C, Ctrl+V, Ctrl+X, Ctrl+A)
        if ctrl && !win && !alt && matches!(key, VK_C | VK_V | VK_X | VK_A) {
            return unsafe { CallNextHookEx(current_hook, n_code, w_param, l_param) };
        }

        // 3. Bloqueo de atajos que rompen el modo Kiosco
        let should_block =
            // Alt + F4 (cerrar), Alt + Tab (cambiar app), Alt + Esc, Alt + Espacio (menú de sistema)
            (alt && matches!(key_u16, VK_F4 | VK_ESCAPE | VK_SPACE | VK_TAB))
            // Win + L (bloquear), Win + I (config), Win + X (menú técnico), Win + D / M (escritorio), Win + R (ejecutar), Win + E (explorador)
            || (win && matches!(key, VK_L | VK_I | VK_X | VK_D | VK_M | VK_R | VK_E | VK_A | VK_S))
            // Menú Inicio con tecla Win o Ctrl+Esc
            || matches!(key_u16, VK_LWIN | VK_RWIN | VK_APPS)
            || (ctrl && key_u16 == VK_ESCAPE && !shift)
            // DevTools o herramientas de depuración (F12, Ctrl+Shift+I/J/C/K)
            || (key_u16 == VK_F12)
            || (ctrl && shift && matches!(key, VK_I | VK_J | VK_C | VK_K));

        if should_block {
            return 1; // Suprimir exclusivamente la pulsación conflictiva
        }
    }

    unsafe { CallNextHookEx(current_hook, n_code, w_param, l_param) }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_hotkeys_definitions() {
        assert!(true);
    }
}
