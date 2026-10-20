mod config;
mod hardware;
mod inactivity;
mod state;
mod vault;

use config::{load_config, save_config, verify_password, AppConfig};
use hardware::{detect_hardware, SystemSpecs};
use inactivity::get_system_idle_seconds;
use state::{build_ui_state, ZenitUiState};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use vault::{delete_vault_video, import_video_to_vault, list_vault_videos, VaultVideo};

static IS_IN_TRY_PC_MODE: AtomicBool = AtomicBool::new(false);
static IS_SCREENSAVER_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InactivityStatus {
    pub idle_seconds: u64,
    pub timeout_seconds: u64,
    pub is_screensaver: bool,
    pub is_in_try_pc_mode: bool,
}

#[tauri::command]
fn get_inactivity_status() -> InactivityStatus {
    let cfg = load_config();
    InactivityStatus {
        idle_seconds: get_system_idle_seconds(),
        timeout_seconds: cfg.inactivity_timeout_sec,
        is_screensaver: IS_SCREENSAVER_ACTIVE.load(Ordering::SeqCst),
        is_in_try_pc_mode: IS_IN_TRY_PC_MODE.load(Ordering::SeqCst),
    }
}

#[tauri::command]
fn get_zenit_ui_state() -> ZenitUiState {
    build_ui_state()
}

#[tauri::command]
fn get_system_specs() -> SystemSpecs {
    let mut specs = detect_hardware();
    let cfg = load_config();

    if let Some(ref brand) = cfg.brand_override {
        if !brand.trim().is_empty() { specs.brand = brand.clone(); }
    }
    if let Some(ref model) = cfg.model_override {
        if !model.trim().is_empty() { specs.model = model.clone(); }
    }
    if let Some(ref proc) = cfg.processor_override {
        if !proc.trim().is_empty() { specs.processor = proc.clone(); }
    }
    if let Some(ref ram) = cfg.ram_override {
        if !ram.trim().is_empty() { specs.ram = ram.clone(); }
    }
    if let Some(ref storage) = cfg.storage_override {
        if !storage.trim().is_empty() { specs.storage = storage.clone(); }
    }
    if let Some(ref display) = cfg.display_override {
        if !display.trim().is_empty() { specs.display = display.clone(); }
    }
    if let Some(ref gpu) = cfg.gpu_override {
        if !gpu.trim().is_empty() { specs.gpu = gpu.clone(); }
    }
    if let Some(ref os) = cfg.os_override {
        if !os.trim().is_empty() { specs.os = os.clone(); }
    }
    if let Some(ref battery) = cfg.battery_override {
        if !battery.trim().is_empty() { specs.battery = battery.clone(); }
    }

    specs
}

#[tauri::command]
fn get_raw_hardware_specs() -> SystemSpecs {
    detect_hardware()
}

#[tauri::command]
fn get_app_config() -> AppConfig {
    load_config()
}

#[tauri::command]
fn save_app_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    save_config(&config)?;
    // Broadcast newly computed state to all windows
    let new_state = build_ui_state();
    let _ = app.emit("zenit://state-changed", new_state);
    Ok(())
}

#[tauri::command]
fn complete_quick_setup(app: AppHandle, mut config: AppConfig) -> Result<(), String> {
    config.setup_completed = true;
    save_config(&config)?;
    let new_state = build_ui_state();
    let _ = app.emit("zenit://state-changed", new_state);
    Ok(())
}

#[tauri::command]
fn check_admin_password(password: String) -> bool {
    verify_password(&password)
}

#[tauri::command]
fn try_pc_minimize(app: AppHandle) -> Result<(), String> {
    IS_IN_TRY_PC_MODE.store(true, Ordering::SeqCst);

    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.minimize();
    }

    if let Some(return_win) = app.get_webview_window("return_window") {
        if let Ok(Some(monitor)) = return_win.primary_monitor() {
            let screen_size = monitor.size();
            let win_width = 240;
            let win_height = 90;

            // Position at vertical center on right edge
            let x = (screen_size.width as i32) - win_width - 15;
            let y = ((screen_size.height as i32) - win_height) / 2;

            let _ = return_win.set_size(PhysicalSize::new(win_width as u32, win_height as u32));
            let _ = return_win.set_position(PhysicalPosition::new(x, y));
        }

        let _ = return_win.set_always_on_top(true);
        let _ = return_win.show();
        let _ = return_win.set_focus();
    }
    Ok(())
}

#[tauri::command]
fn restore_main_window(app: AppHandle) -> Result<(), String> {
    IS_IN_TRY_PC_MODE.store(false, Ordering::SeqCst);

    if let Some(return_win) = app.get_webview_window("return_window") {
        let _ = return_win.hide();
    }

    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.unminimize();
        let _ = main_win.show();
        let _ = main_win.set_fullscreen(true);
        let _ = main_win.set_always_on_top(true);
        let _ = main_win.set_focus();
    }
    Ok(())
}

#[tauri::command]
fn emergency_exit() {
    std::process::exit(0);
}

#[tauri::command]
fn emergency_restart(app: AppHandle) {
    if let Ok(current_exe) = std::env::current_exe() {
        let _ = std::process::Command::new(current_exe).spawn();
    }
    app.exit(0);
}

#[tauri::command]
fn import_video(source_path: String) -> Result<VaultVideo, String> {
    import_video_to_vault(&source_path)
}

#[tauri::command]
fn get_vault_videos() -> Vec<VaultVideo> {
    list_vault_videos()
}

#[tauri::command]
fn remove_vault_video(filename: String) -> Result<(), String> {
    delete_vault_video(&filename)
}

pub fn run() {
    // Enforce 1:1 display scale immunity and disable zoom gestures/hotkeys in WebView2
    unsafe {
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--force-device-scale-factor=1 --disable-pinch --disable-features=msWebview2ZoomHotkeys",
        );
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Main Window setup
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.set_fullscreen(true);
                let _ = main_win.set_always_on_top(true);
                let _ = main_win.set_focus();
            }

            // Global Shortcuts for emergency exit and restart
            let handle_exit = app.handle().clone();
            let handle_restart = app.handle().clone();

            let exit_shortcut: Shortcut = "Control+Shift+Alt+KeyZ".parse().unwrap();
            let restart_shortcut: Shortcut = "Control+Shift+Alt+KeyR".parse().unwrap();

            let _ = app.global_shortcut().on_shortcut(exit_shortcut, move |_app, _shortcut, _event| {
                handle_exit.exit(0);
            });

            let _ = app.global_shortcut().on_shortcut(restart_shortcut, move |_app, _shortcut, _event| {
                if let Ok(current_exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(current_exe).spawn();
                }
                handle_restart.exit(0);
            });

            // Native Rust Background Inactivity Watcher (Zero JS Overhead)
            let watcher_handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let idle_seconds = get_system_idle_seconds();
                    let cfg = load_config();
                    let timeout = cfg.inactivity_timeout_sec;

                    // 1. Screensaver state toggle
                    let was_screensaver = IS_SCREENSAVER_ACTIVE.load(Ordering::SeqCst);
                    if !was_screensaver && idle_seconds >= timeout {
                        IS_SCREENSAVER_ACTIVE.store(true, Ordering::SeqCst);
                        let _ = watcher_handle.emit("zenit://screensaver-change", true);
                    } else if was_screensaver && idle_seconds < 2 {
                        IS_SCREENSAVER_ACTIVE.store(false, Ordering::SeqCst);
                        let _ = watcher_handle.emit("zenit://screensaver-change", false);
                    }

                    // 2. Try-PC Auto-restore after 120s of no desktop activity
                    if IS_IN_TRY_PC_MODE.load(Ordering::SeqCst) && idle_seconds >= 120 {
                        IS_IN_TRY_PC_MODE.store(false, Ordering::SeqCst);
                        let _ = restore_main_window(watcher_handle.clone());
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_zenit_ui_state,
            get_inactivity_status,
            get_system_specs,
            get_raw_hardware_specs,
            get_app_config,
            save_app_config,
            complete_quick_setup,
            check_admin_password,
            try_pc_minimize,
            restore_main_window,
            emergency_exit,
            emergency_restart,
            import_video,
            get_vault_videos,
            remove_vault_video,
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación Zenit");
}
