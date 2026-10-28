use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{
    GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_HIGHEST,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
    VK_APPS, VK_CONTROL, VK_ESCAPE, VK_F12, VK_F4, VK_LCONTROL, VK_LMENU, VK_LSHIFT,
    VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT, VK_SPACE, VK_TAB,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostQuitMessage, SetWindowsHookExW, UnhookWindowsHookEx,
    HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, LLKHF_ALTDOWN, MSG, WH_KEYBOARD_LL, WM_HOTKEY,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

static WIN_DOWN: AtomicBool = AtomicBool::new(false);
static CTRL_DOWN: AtomicBool = AtomicBool::new(false);
static SHIFT_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);

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

    // Regla de Oro: En KeyUp SIEMPRE actualizamos y dejamos pasar el evento
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
            println!("  -> [PERMITIDO] Ctrl + Shift + Esc (Task Manager)");
            return unsafe { CallNextHookEx(current_hook, n_code, w_param, l_param) };
        }

        // 2. Permitir atajos normales de edición de texto (Ctrl+C, Ctrl+V, Ctrl+X, Ctrl+A)
        if ctrl && !win && !alt && matches!(key, VK_C | VK_V | VK_X | VK_A) {
            println!("  -> [PERMITIDO] Atajo de edición (Ctrl + C/V/X/A)");
            return unsafe { CallNextHookEx(current_hook, n_code, w_param, l_param) };
        }

        // 3. Bloqueo de atajos que rompen el modo Kiosco
        let mut reason: Option<&str> = None;

        if alt && matches!(key_u16, VK_F4 | VK_ESCAPE | VK_SPACE | VK_TAB) {
            reason = Some(match key_u16 {
                VK_F4 => "Alt + F4",
                VK_TAB => "Alt + Tab",
                VK_SPACE => "Alt + Space",
                VK_ESCAPE => "Alt + Esc",
                _ => "Alt + Combinación",
            });
        } else if win && !matches!(key_u16, VK_LWIN | VK_RWIN) && matches!(key, VK_L | VK_I | VK_X | VK_D | VK_M | VK_R | VK_E | VK_A | VK_S) {
            reason = Some("Win + Atajo de Sistema");
        } else if key_u16 == VK_APPS {
            reason = Some("Menú Apps");
        } else if ctrl && key_u16 == VK_ESCAPE && !shift {
            reason = Some("Ctrl + Esc (Menú Inicio)");
        } else if key_u16 == VK_F12 {
            reason = Some("F12 (DevTools)");
        } else if ctrl && shift && matches!(key, VK_I | VK_J | VK_C | VK_K) {
            reason = Some("Ctrl + Shift + DevShortcut");
        }

        if let Some(r) = reason {
            println!("  [BLOQUEADO] {}", r);
            return 1;
        }
    }

    unsafe { CallNextHookEx(current_hook, n_code, w_param, l_param) }
}

fn main() {
    println!("========================================================");
    println!("     ZENIT - PROBADOR DE GUARDIÁN DE TECLADO / KIOSCO    ");
    println!("========================================================");
    println!("Este test corre exactamente la misma lógica de hotkeys.rs");
    println!();
    println!("Prueba las siguientes acciones:");
    println!("  1. Presiona [Alt + F4]       -> Debería salir [BLOQUEADO] Alt + F4");
    println!("  2. Presiona [Alt + Tab]      -> Debería salir [BLOQUEADO] Alt + Tab");
    println!("  3. Presiona [Alt + Espacio]  -> Debería salir [BLOQUEADO] Alt + Space");
    println!("  4. Presiona [Espacio] solo   -> NO se debe bloquear");
    println!("  5. Presiona [Tecla Win]      -> Debería salir [BLOQUEADO] Tecla Windows");
    println!("  6. Presiona [Win + L/D/E/R]  -> Debería salir [BLOQUEADO] Win + Atajo");
    println!("  7. Presiona [F12]            -> Debería salir [BLOQUEADO] F12");
    println!("  8. Presiona [Ctrl + Shift + Esc] -> Debería salir [PERMITIDO]");
    println!("  9. Presiona [Ctrl + C]       -> Debería salir [PERMITIDO]");
    println!();
    println!("Para SALIR de esta prueba:");
    println!("  - Presiona [Ctrl + Shift + Alt + Z]  (Atajo de kernel)");
    println!("  - O presiona [Ctrl + C] en esta consola");
    println!("========================================================");
    println!("Escuchando eventos de teclado... (escribe o prueba atajos)");

    unsafe {
        SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);

        const HOTKEY_EXIT_ID: i32 = 9101;
        let modifiers = MOD_CONTROL | MOD_SHIFT | MOD_ALT | MOD_NOREPEAT;

        let ok_z = RegisterHotKey(std::ptr::null_mut(), HOTKEY_EXIT_ID, modifiers, b'Z' as u32);
        if ok_z != 0 {
            println!("[OK] Kernel HotKey registrado: Ctrl+Shift+Alt+Z para salir.");
        }

        let h_instance = GetModuleHandleW(std::ptr::null());
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            h_instance,
            0,
        );

        if hook.is_null() {
            eprintln!("[ERROR] No se pudo instalar el hook de teclado!");
            return;
        }

        HOOK_HANDLE.store(hook as isize, Ordering::SeqCst);
        println!("[OK] Hook de teclado de bajo nivel ACTIVO.\n");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_HOTKEY && msg.wParam as i32 == HOTKEY_EXIT_ID {
                println!("\n[Zenit] Salida rápida ejecutada (Ctrl+Shift+Alt+Z). Terminando prueba.");
                PostQuitMessage(0);
                break;
            }
        }

        UnhookWindowsHookEx(hook);
        UnregisterHotKey(std::ptr::null_mut(), HOTKEY_EXIT_ID);
    }
    println!("Prueba finalizada. Hook desinstalado.");
}
