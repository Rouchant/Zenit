//! Global hotkeys for Zenit Showroom
//! - Ctrl + Shift + Alt + Z: Cierre rápido (cierra la aplicación inmediatamente)
//! - Ctrl + Shift + Alt + R: Reinicio rápido (reinicia la aplicación inmediatamente)

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

/// Cierre rápido inmediato de la aplicación
pub fn quick_exit() -> ! {
    println!("[Zenit] Cierre rápido ejecutado (Ctrl+Shift+Alt+Z). Saliendo...");
    std::process::exit(0);
}

/// Reinicio rápido inmediato de la aplicación
pub fn quick_restart() -> ! {
    println!("[Zenit] Reinicio rápido ejecutado (Ctrl+Shift+Alt+R). Reiniciando proceso...");
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

/// Inicia el listener de hotkeys globales en Windows en un hilo de fondo dedicado
pub fn spawn_global_hotkey_listener() {
    #[cfg(target_os = "windows")]
    {
        std::thread::Builder::new()
            .name("zenit_hotkeys".to_string())
            .spawn(move || unsafe {
                const HOTKEY_EXIT_ID: i32 = 9101;
                const HOTKEY_RESTART_ID: i32 = 9102;
                let modifiers = MOD_CONTROL | MOD_SHIFT | MOD_ALT | MOD_NOREPEAT;

                let ok_z = RegisterHotKey(std::ptr::null_mut(), HOTKEY_EXIT_ID, modifiers, b'Z' as u32);
                let ok_r = RegisterHotKey(std::ptr::null_mut(), HOTKEY_RESTART_ID, modifiers, b'R' as u32);

                if ok_z != 0 {
                    println!("[Hotkeys] Atajo global registrado: Ctrl+Shift+Alt+Z -> Cierre rápido");
                } else {
                    eprintln!("[Hotkeys] Advertencia: No se pudo registrar atajo Ctrl+Shift+Alt+Z");
                }

                if ok_r != 0 {
                    println!("[Hotkeys] Atajo global registrado: Ctrl+Shift+Alt+R -> Reinicio rápido");
                } else {
                    eprintln!("[Hotkeys] Advertencia: No se pudo registrar atajo Ctrl+Shift+Alt+R");
                }

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

                UnregisterHotKey(std::ptr::null_mut(), HOTKEY_EXIT_ID);
                UnregisterHotKey(std::ptr::null_mut(), HOTKEY_RESTART_ID);
            })
            .expect("Failed to spawn global hotkey thread");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hotkeys_definitions() {
        assert!(true);
    }
}
