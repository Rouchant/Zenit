pub mod domains;

use domains::hardware::detect_hardware;
use domains::kiosk::get_system_idle_seconds;
use domains::showcase::{build_ui_state, load_config, save_config, verify_password};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

slint::include_modules!();

static IS_IN_TRY_PC_MODE: AtomicBool = AtomicBool::new(false);
static IS_SCREENSAVER_ACTIVE: AtomicBool = AtomicBool::new(false);

fn populate_ui_state(window: &MainWindow) {
    let state = build_ui_state();
    let cfg = load_config();

    window.set_brand_name(state.brand_name.clone().into());
    let model_display = cfg.model_override.clone().unwrap_or_else(|| {
        if !state.right_specs.is_empty() {
            state.right_specs.iter()
                .find(|s| s.id == "card-model")
                .map(|s| s.value.clone())
                .unwrap_or_else(|| "PC Showroom".to_string())
        } else {
            "PC Showroom".to_string()
        }
    });
    window.set_model_name(model_display.into());

    let retail_upper = if cfg.store_theme.is_empty() || cfg.store_theme == "default" {
        "SHOWROOM".to_string()
    } else {
        cfg.store_theme.to_uppercase()
    };
    window.set_retail_name(retail_upper.clone().into());

    let card_p = if cfg.price_card.trim().is_empty() {
        "$1.299.990".to_string()
    } else {
        cfg.price_card.clone()
    };
    window.set_card_price(card_p.into());
    window.set_card_label(format!("Tarjeta {}", retail_upper).into());

    let normal_p = if cfg.price_all_methods.trim().is_empty() {
        "$1.349.990".to_string()
    } else {
        cfg.price_all_methods.clone()
    };
    window.set_normal_price(normal_p.into());
    window.set_normal_label("Otros medios de pago".into());

    let sku_code = if cfg.sku.trim().is_empty() {
        "8493021".to_string()
    } else {
        cfg.sku.clone()
    };
    window.set_sku_code(sku_code.into());

    // Populate Left Column Specs
    let left_items: Vec<SpecItem> = state.left_specs.into_iter().map(|s| {
        let icon_img = slint::Image::load_from_path(std::path::Path::new(&s.icon)).unwrap_or_default();
        SpecItem {
            category: s.label.to_uppercase().into(),
            value: s.value.into(),
            subvalue: s.sub_value.unwrap_or_default().into(),
            tag: s.tag.unwrap_or_default().into(),
            icon: icon_img,
        }
    }).collect();
    window.set_left_specs(ModelRc::new(VecModel::from(left_items)));

    // Populate Right Column Specs
    let right_items: Vec<SpecItem> = state.right_specs.into_iter().map(|s| {
        let icon_img = slint::Image::load_from_path(std::path::Path::new(&s.icon)).unwrap_or_default();
        SpecItem {
            category: s.label.to_uppercase().into(),
            value: s.value.into(),
            subvalue: s.sub_value.unwrap_or_default().into(),
            tag: s.tag.unwrap_or_default().into(),
            icon: icon_img,
        }
    }).collect();
    window.set_right_specs(ModelRc::new(VecModel::from(right_items)));

    // Quick Setup Initial State
    window.set_is_quick_setup_open(!cfg.setup_completed);
    window.set_setup_brand(state.brand_name.into());
    window.set_setup_model(cfg.model_override.unwrap_or_default().into());
    window.set_setup_card_price(cfg.price_card.into());
    window.set_setup_normal_price(cfg.price_all_methods.into());
    window.set_setup_sku(cfg.sku.into());
    window.set_setup_retail(cfg.store_theme.into());
}

pub fn run() {
    let main_window = MainWindow::new().expect("Failed to initialize Slint MainWindow");

    // Make window visible and fullscreen in kiosk showroom mode
    main_window.show().expect("Failed to show Slint window");
    main_window.window().set_fullscreen(true);

    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            FindWindowA, GetSystemMetrics, SetWindowPos, HWND_TOP, SM_CXSCREEN, SM_CYSCREEN,
            SWP_SHOWWINDOW,
        };
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        println!("[Showroom] Resolucion detectada de pantalla: {}x{}", screen_w, screen_h);
        main_window.set_screen_width(screen_w as f32);
        main_window.set_screen_height(screen_h as f32);

        let title = b"Showroom PC - Modo Exhibici\xc3\xb3n\0";
        let hwnd = FindWindowA(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            SetWindowPos(hwnd, HWND_TOP, 0, 0, screen_w, screen_h, SWP_SHOWWINDOW);
        }
    }

    populate_ui_state(&main_window);

    // Initial Live Clock
    let now = chrono::Local::now();
    main_window.set_live_clock(now.format("%H:%M:%S").to_string().into());

    // Initialize libmpv Showroom Video Player with native Slint Timer
    let video_timer = slint::Timer::default();
    let video_path = domains::media::resolve_showroom_video_path();
    let video_path_str = video_path.to_string_lossy().to_string();

    let player_shared: Arc<Mutex<Option<domains::media::SlintMpvPlayer>>> =
        Arc::new(Mutex::new(None));

    match domains::media::SlintMpvPlayer::new(&video_path_str, 854, 480) {
        Ok(player) => {
            println!("[libmpv] Reproductor inicializado con video: {}", video_path_str);
            main_window.set_has_video_stream(true);
            *player_shared.lock().unwrap() = Some(player);
            let vh = main_window.as_weak();
            let player_for_timer = player_shared.clone();
            video_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(33), move || {
                if !IS_IN_TRY_PC_MODE.load(Ordering::SeqCst) {
                    if let Ok(mut guard) = player_for_timer.try_lock() {
                        if let Some(ref mut p) = *guard {
                            match p.render_frame_buffer() {
                                Ok(buffer) => {
                                    if let Some(ui) = vh.upgrade() {
                                        ui.set_video_frame(slint::Image::from_rgba8(buffer));
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[libmpv render error] {}", e);
                                }
                            }
                        }
                    }
                }
            });
        }
        Err(err) => {
            println!("[libmpv] {}", err);
            main_window.set_has_video_stream(false);
        }
    }

    // Live Clock Timer (updates every 1s)
    let clock_timer = slint::Timer::default();
    let clock_handle = main_window.as_weak();
    clock_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_secs(1), move || {
        if let Some(ui) = clock_handle.upgrade() {
            let current = chrono::Local::now();
            ui.set_live_clock(current.format("%H:%M:%S").to_string().into());
        }
    });

    // 4-Tap Admin Trigger in top-right corner
    let tap_state = Rc::new(RefCell::new((0u32, Instant::now())));
    let tap_handle = main_window.as_weak();
    let tap_state_clone = tap_state.clone();
    main_window.on_corner_tap_clicked(move || {
        let mut st = tap_state_clone.borrow_mut();
        let now = Instant::now();
        if now.duration_since(st.1).as_millis() < 2000 {
            st.0 += 1;
        } else {
            st.0 = 1;
        }
        st.1 = now;

        if st.0 >= 4 {
            st.0 = 0;
            if let Some(ui) = tap_handle.upgrade() {
                ui.set_is_admin_open(true);
                ui.set_is_admin_authenticated(false);
                ui.set_admin_error_msg("".into());
                ui.set_admin_password_input("".into());
            }
        }
    });

    // Close Admin Panel
    let close_admin_handle = main_window.as_weak();
    main_window.on_close_admin_clicked(move || {
        if let Some(ui) = close_admin_handle.upgrade() {
            ui.set_is_admin_open(false);
            ui.set_is_admin_authenticated(false);
            ui.set_admin_error_msg("".into());
        }
    });

    // Verify Admin Password
    let pwd_handle = main_window.as_weak();
    main_window.on_check_admin_password(move |pwd| {
        if let Some(ui) = pwd_handle.upgrade() {
            if verify_password(&pwd) {
                ui.set_is_admin_authenticated(true);
                ui.set_admin_error_msg("".into());
            } else {
                ui.set_admin_error_msg("Contraseña incorrecta. Intente nuevamente.".into());
            }
        }
    });

    // Auto-detect Hardware for Quick Setup
    let detect_handle = main_window.as_weak();
    main_window.on_auto_detect_clicked(move || {
        if let Some(ui) = detect_handle.upgrade() {
            let hw = detect_hardware();
            ui.set_setup_brand(hw.brand.into());
            ui.set_setup_model(hw.model.into());
        }
    });

    // Save Quick Setup Form
    let save_setup_handle = main_window.as_weak();
    main_window.on_save_quick_setup(move |brand, model, card_p, normal_p, sku, retail| {
        if let Some(ui) = save_setup_handle.upgrade() {
            let mut cfg = load_config();
            if !brand.trim().is_empty() {
                cfg.brand_override = Some(brand.to_string());
            }
            if !model.trim().is_empty() {
                cfg.model_override = Some(model.to_string());
            }
            cfg.price_card = card_p.to_string();
            cfg.price_all_methods = normal_p.to_string();
            cfg.sku = sku.to_string();
            if !retail.trim().is_empty() {
                cfg.store_theme = retail.to_string();
            }
            cfg.setup_completed = true;
            let _ = save_config(&cfg);

            ui.set_is_quick_setup_open(false);
            populate_ui_state(&ui);
        }
    });

    // Floating "Ver Detalles" Button Widget
    let floating_btn = Rc::new(FloatingDetailsButton::new().expect("Failed to initialize FloatingDetailsButton"));
    let restore_mw_handle = main_window.as_weak();
    let restore_fb_handle = floating_btn.as_weak();

    floating_btn.on_restore_clicked(move || {
        IS_IN_TRY_PC_MODE.store(false, Ordering::SeqCst);
        if let Some(fb) = restore_fb_handle.upgrade() {
            let _ = fb.hide();
        }
        if let Some(mw) = restore_mw_handle.upgrade() {
            mw.window().set_fullscreen(true);
            #[cfg(target_os = "windows")]
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    FindWindowA, SetForegroundWindow, ShowWindow, SW_RESTORE,
                };
                let title = b"Showroom PC - Modo Exhibici\xc3\xb3n\0";
                let hwnd = FindWindowA(std::ptr::null(), title.as_ptr());
                if !hwnd.is_null() {
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                }
            }
        }
    });

    // Try PC Mode
    let try_pc_mw_handle = main_window.as_weak();
    let try_pc_fb_handle = floating_btn.as_weak();
    main_window.on_try_pc_clicked(move || {
        IS_IN_TRY_PC_MODE.store(true, Ordering::SeqCst);
        if let Some(fb) = try_pc_fb_handle.upgrade() {
            let _ = fb.show();
            #[cfg(target_os = "windows")]
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    FindWindowA, GetSystemMetrics, SetWindowPos, HWND_TOPMOST, SM_CXSCREEN,
                    SM_CYSCREEN, SWP_SHOWWINDOW,
                };
                let btn_title = b"Zenit - Ver Detalles\0";
                let btn_hwnd = FindWindowA(std::ptr::null(), btn_title.as_ptr());
                if !btn_hwnd.is_null() {
                    let screen_w = GetSystemMetrics(SM_CXSCREEN);
                    let screen_h = GetSystemMetrics(SM_CYSCREEN);
                    let w = 220;
                    let h = 60;
                    let x = screen_w - w - 40;
                    let y = screen_h - h - 60;
                    SetWindowPos(btn_hwnd, HWND_TOPMOST, x, y, w, h, SWP_SHOWWINDOW);
                }
            }
        }

        if let Some(_ui) = try_pc_mw_handle.upgrade() {
            // Under Windows, minimize main window so user can interact with the desktop
            #[cfg(target_os = "windows")]
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowA, ShowWindow, SW_MINIMIZE};
                let title = b"Showroom PC - Modo Exhibici\xc3\xb3n\0";
                let hwnd = FindWindowA(std::ptr::null(), title.as_ptr());
                if !hwnd.is_null() {
                    ShowWindow(hwnd, SW_MINIMIZE);
                }
            }
        }
    });

    // Screensaver Dismiss
    let dismiss_handle = main_window.as_weak();
    let player_dismiss = player_shared.clone();
    main_window.on_dismiss_screensaver(move || {
        if let Some(ui) = dismiss_handle.upgrade() {
            ui.set_is_screensaver_active(false);
            IS_SCREENSAVER_ACTIVE.store(false, Ordering::SeqCst);
        }
        let landing_video = domains::media::resolve_showroom_video_path();
        if let Ok(mut guard) = player_dismiss.lock() {
            if let Some(ref mut p) = *guard {
                p.change_video(&landing_video.to_string_lossy());
            }
        }
    });

    // Emergency Exit
    main_window.on_emergency_exit_clicked(|| {
        std::process::exit(0);
    });

    // Emergency Restart
    main_window.on_emergency_restart_clicked(|| {
        if let Ok(current_exe) = std::env::current_exe() {
            let _ = std::process::Command::new(current_exe).spawn();
        }
        std::process::exit(0);
    });

    // Background Inactivity Watcher (Zero JS Overhead, 100% Native Rust)
    let watcher_handle = main_window.as_weak();
    let watcher_fb_handle = floating_btn.as_weak();
    let player_watcher = player_shared.clone();
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
                let promo_video = domains::media::resolve_video_path(&cfg.inactivity_video_path);
                if promo_video.exists() {
                    if let Ok(mut guard) = player_watcher.lock() {
                        if let Some(ref mut p) = *guard {
                            p.change_video(&promo_video.to_string_lossy());
                        }
                    }
                }
                let wh = watcher_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = wh.upgrade() {
                        ui.set_is_screensaver_active(true);
                    }
                });
            } else if was_screensaver && idle_seconds < 2 {
                IS_SCREENSAVER_ACTIVE.store(false, Ordering::SeqCst);
                let landing_video = domains::media::resolve_showroom_video_path();
                if let Ok(mut guard) = player_watcher.lock() {
                    if let Some(ref mut p) = *guard {
                        p.change_video(&landing_video.to_string_lossy());
                    }
                }
                let wh = watcher_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = wh.upgrade() {
                        ui.set_is_screensaver_active(false);
                    }
                });
            }

            // 2. Try-PC Auto-restore after 120s of no desktop activity
            if IS_IN_TRY_PC_MODE.load(Ordering::SeqCst) && idle_seconds >= 120 {
                IS_IN_TRY_PC_MODE.store(false, Ordering::SeqCst);
                let landing_video = domains::media::resolve_showroom_video_path();
                if let Ok(mut guard) = player_watcher.lock() {
                    if let Some(ref mut p) = *guard {
                        p.change_video(&landing_video.to_string_lossy());
                    }
                }
                let wh = watcher_handle.clone();
                let fbh = watcher_fb_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(fb) = fbh.upgrade() {
                        let _ = fb.hide();
                    }
                    if let Some(ui) = wh.upgrade() {
                        ui.window().set_fullscreen(true);
                        ui.set_is_screensaver_active(false);
                        #[cfg(target_os = "windows")]
                        unsafe {
                            use windows_sys::Win32::UI::WindowsAndMessaging::{
                                FindWindowA, SetForegroundWindow, ShowWindow, SW_RESTORE,
                            };
                            let title = b"Showroom PC - Modo Exhibici\xc3\xb3n\0";
                            let hwnd = FindWindowA(std::ptr::null(), title.as_ptr());
                            if !hwnd.is_null() {
                                ShowWindow(hwnd, SW_RESTORE);
                                SetForegroundWindow(hwnd);
                            }
                        }
                    }
                });
            }
        }
    });

    // Ensure true fullscreen kiosk mode
    main_window.window().set_fullscreen(true);

    // Start native event loop
    main_window.run().expect("Error running Slint application");
}
