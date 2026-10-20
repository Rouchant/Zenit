pub mod domains;

use domains::hardware::detect_hardware;
use domains::kiosk::get_system_idle_seconds;
use domains::showcase::{build_ui_state, load_config, save_config, verify_password};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::HWND;

slint::include_modules!();

static IS_IN_TRY_PC_MODE: AtomicBool = AtomicBool::new(false);
static IS_SCREENSAVER_ACTIVE: AtomicBool = AtomicBool::new(false);
static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
static FLOATING_HWND: AtomicIsize = AtomicIsize::new(0);

fn populate_vault_videos(window: &MainWindow) {
    domains::media::vault::seed_vault_defaults();
    let videos = domains::media::vault::list_vault_videos();
    let items: Vec<VaultVideoItem> = videos.into_iter().map(|v| {
        VaultVideoItem {
            name: v.name.into(),
            filename: v.filename.into(),
            path: v.path.into(),
            size_str: format!("{:.1} MB", v.size_mb).into(),
        }
    }).collect();
    window.set_vault_videos(ModelRc::new(VecModel::from(items)));
}

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
    window.set_model_name(model_display.clone().into());

    let card_trim = cfg.price_card.trim();
    let normal_trim = cfg.price_all_methods.trim();
    let sku_trim = cfg.sku.trim();
    let store_trim = cfg.store_theme.trim();

    let retail_display = if store_trim.is_empty() || store_trim.eq_ignore_ascii_case("default") {
        "".to_string()
    } else {
        store_trim.to_uppercase()
    };
    window.set_retail_name(retail_display.clone().into());

    let card_label = if !retail_display.is_empty() {
        format!("Tarjeta {}", retail_display)
    } else {
        "Precio Tarjeta".to_string()
    };
    window.set_card_label(card_label.into());
    window.set_card_price(card_trim.into());

    let normal_label = if card_trim.is_empty() {
        "Todo medio de pago".to_string()
    } else {
        "Otros medios de pago".to_string()
    };
    window.set_normal_label(normal_label.into());
    window.set_normal_price(normal_trim.into());

    window.set_sku_code(sku_trim.into());

    // Populate Left Column Specs
    let left_items: Vec<SpecItem> = state.left_specs.iter().map(|s| {
        let icon_img = slint::Image::load_from_path(std::path::Path::new(&s.icon)).unwrap_or_default();
        SpecItem {
            category: s.label.to_uppercase().into(),
            value: s.value.clone().into(),
            subvalue: s.sub_value.clone().unwrap_or_default().into(),
            tag: s.tag.clone().unwrap_or_default().into(),
            icon: icon_img,
        }
    }).collect();
    window.set_left_specs(ModelRc::new(VecModel::from(left_items)));

    // Populate Right Column Specs
    let right_items: Vec<SpecItem> = state.right_specs.iter().map(|s| {
        let icon_img = slint::Image::load_from_path(std::path::Path::new(&s.icon)).unwrap_or_default();
        SpecItem {
            category: s.label.to_uppercase().into(),
            value: s.value.clone().into(),
            subvalue: s.sub_value.clone().unwrap_or_default().into(),
            tag: s.tag.clone().unwrap_or_default().into(),
            icon: icon_img,
        }
    }).collect();
    window.set_right_specs(ModelRc::new(VecModel::from(right_items)));

    // Quick Setup Initial State
    window.set_is_quick_setup_open(!cfg.setup_completed);
    window.set_setup_brand(state.brand_name.clone().into());
    window.set_setup_model(cfg.model_override.clone().unwrap_or_default().into());
    window.set_setup_card_price(cfg.price_card.clone().into());
    window.set_setup_normal_price(cfg.price_all_methods.clone().into());
    window.set_setup_sku(cfg.sku.clone().into());
    window.set_setup_retail(cfg.store_theme.clone().into());

    // Admin Configuration Initial State
    let hw = detect_hardware();

    let brand_val = cfg.brand_override.clone().unwrap_or_else(|| {
        if !hw.brand.is_empty() { hw.brand.clone() } else { state.brand_name.clone() }
    });
    window.set_cfg_brand(brand_val.into());

    let model_val = cfg.model_override.clone().unwrap_or_else(|| {
        if !hw.model.is_empty() { hw.model.clone() } else { model_display.clone() }
    });
    window.set_cfg_model(model_val.into());

    window.set_cfg_retail(cfg.store_theme.clone().into());
    window.set_cfg_card_price(cfg.price_card.clone().into());
    window.set_cfg_normal_price(cfg.price_all_methods.clone().into());
    window.set_cfg_sku(cfg.sku.clone().into());

    // Fall back to recognized hardware specs if user hasn't set an override
    let cpu_val = cfg.processor_override.clone().unwrap_or_else(|| {
        state.left_specs.iter()
            .find(|s| s.id == "card-processor")
            .map(|s| s.value.clone())
            .unwrap_or(hw.processor.clone())
    });
    window.set_cfg_cpu(cpu_val.into());

    let ram_val = cfg.ram_override.clone().unwrap_or_else(|| {
        state.left_specs.iter()
            .find(|s| s.id == "card-ram")
            .map(|s| s.value.clone())
            .unwrap_or(hw.ram.clone())
    });
    window.set_cfg_ram(ram_val.into());

    let storage_val = cfg.storage_override.clone().unwrap_or_else(|| {
        state.left_specs.iter()
            .find(|s| s.id == "card-storage")
            .map(|s| s.value.clone())
            .unwrap_or(hw.storage.clone())
    });
    window.set_cfg_storage(storage_val.into());

    let gpu_val = cfg.gpu_override.clone().unwrap_or_else(|| {
        state.right_specs.iter()
            .find(|s| s.id == "card-gpu")
            .map(|s| s.value.clone())
            .unwrap_or(hw.gpu.clone())
    });
    window.set_cfg_gpu(gpu_val.into());

    let display_val = cfg.display_override.clone().unwrap_or_else(|| {
        state.right_specs.iter()
            .find(|s| s.id == "card-display")
            .map(|s| s.value.clone())
            .unwrap_or(hw.display.clone())
    });
    window.set_cfg_display(display_val.into());

    let os_val = cfg.os_override.clone().unwrap_or_else(|| {
        state.left_specs.iter()
            .find(|s| s.id == "card-os")
            .map(|s| s.value.clone())
            .unwrap_or(hw.os.clone())
    });
    window.set_cfg_os(os_val.into());

    let battery_val = cfg.battery_override.clone().unwrap_or_else(|| {
        let b = state.right_specs.iter()
            .find(|s| s.id == "card-battery")
            .map(|s| s.value.clone())
            .unwrap_or_else(|| hw.battery.clone());
        if !b.is_empty() && !b.contains("Conectado AC") {
            b
        } else {
            "Hasta 8 horas".to_string()
        }
    });
    window.set_cfg_battery(battery_val.into());

    let others_val = if cfg.other_specs.trim().is_empty() {
        "Wi-Fi 6 / Bluetooth 5.3 / Teclado en Español".to_string()
    } else {
        cfg.other_specs.clone()
    };
    window.set_cfg_others(others_val.into());

    window.set_cfg_central_video(cfg.central_video_path.clone().into());
    window.set_cfg_bg_video(cfg.bg_video_path.clone().into());
    window.set_cfg_inactivity_video_1(cfg.inactivity_video_path.clone().into());
    window.set_cfg_inactivity_video_2(cfg.inactivity_video_path_2.clone().into());
    window.set_cfg_inactivity_video_3(cfg.inactivity_video_path_3.clone().into());
    window.set_cfg_inactivity_timeout(cfg.inactivity_timeout_sec.to_string().into());

    populate_vault_videos(window);
}

pub fn run() {
    let main_window = MainWindow::new().expect("Failed to initialize Slint MainWindow");

    // Make window visible and fullscreen in kiosk showroom mode
    main_window.show().expect("Failed to show Slint window");
    main_window.window().set_fullscreen(true);

    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::Foundation::LPARAM;
        use windows_sys::Win32::System::Threading::GetCurrentThreadId;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            EnumThreadWindows, GetSystemMetrics, SetWindowPos, HWND_TOP, SM_CXSCREEN, SM_CYSCREEN,
            SWP_SHOWWINDOW,
        };

        unsafe extern "system" fn enum_main_win(hwnd: HWND, _: LPARAM) -> i32 {
            if MAIN_HWND.load(Ordering::SeqCst) == 0 {
                MAIN_HWND.store(hwnd as isize, Ordering::SeqCst);
            }
            1
        }

        let thread_id = GetCurrentThreadId();
        EnumThreadWindows(thread_id, Some(enum_main_win), 0);

        let hwnd = MAIN_HWND.load(Ordering::SeqCst) as HWND;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        println!("[Showroom] Resolucion detectada de pantalla: {}x{}, HWND: {:?}", screen_w, screen_h, hwnd);
        main_window.set_screen_width(screen_w as f32);
        main_window.set_screen_height(screen_h as f32);

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
            println!("[libmpv] Reproductor landing inicializado con video: {}", video_path_str);
            main_window.set_has_video_stream(true);
            *player_shared.lock().unwrap() = Some(player);
        }
        Err(err) => {
            println!("[libmpv error] {}", err);
            main_window.set_has_video_stream(false);
        }
    }

    // Fullscreen Background: Use high-performance deep tech gradient
    // Disabling dual-video playback overhead ensures 100% fluid video playback for landing & screensaver.
    main_window.set_has_bg_video(false);

    let vh = main_window.as_weak();
    let player_for_timer = player_shared.clone();
    // 16ms timer delivers silky smooth up to 60 FPS video playback without frame drops
    video_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(16), move || {
        if !IS_IN_TRY_PC_MODE.load(Ordering::SeqCst) {
            // Render Landing / Screensaver video stream (Ultra-fluid 60 FPS)
            if let Ok(mut guard) = player_for_timer.try_lock() {
                if let Some(ref mut p) = *guard {
                    if let Ok(buffer) = p.render_frame_buffer() {
                        if let Some(ui) = vh.upgrade() {
                            ui.set_video_frame(slint::Image::from_rgba8(buffer));
                        }
                    }
                }
            }
        }
    });

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
                populate_ui_state(&ui);
            } else {
                ui.set_admin_error_msg("Contraseña incorrecta. Intente nuevamente.".into());
            }
        }
    });

    // Auto-detect Hardware for Quick Setup & Admin Panel
    let detect_handle = main_window.as_weak();
    main_window.on_auto_detect_clicked(move || {
        if let Some(ui) = detect_handle.upgrade() {
            let hw = detect_hardware();
            ui.set_setup_brand(hw.brand.clone().into());
            ui.set_setup_model(hw.model.clone().into());

            // Also fill Admin inputs
            ui.set_cfg_brand(hw.brand.into());
            ui.set_cfg_model(hw.model.into());
            ui.set_cfg_cpu(hw.processor.into());
            ui.set_cfg_ram(hw.ram.into());
            ui.set_cfg_storage(hw.storage.into());
            ui.set_cfg_gpu(hw.gpu.into());
            ui.set_cfg_display(hw.display.into());
            ui.set_cfg_os(hw.os.into());
            if hw.battery.contains("Conectado AC") || hw.battery.contains("Cargando") {
                if ui.get_cfg_battery().trim().is_empty() {
                    ui.set_cfg_battery("Hasta 8 horas".into());
                }
            } else {
                ui.set_cfg_battery(hw.battery.into());
            }
        }
    });

    // Save Admin Configuration Form (Tienda, Especificaciones, Videos, Sistema)
    let save_admin_handle = main_window.as_weak();
    let landing_player_on_save = player_shared.clone();
    main_window.on_save_admin_config(move |brand, model, retail, card_p, normal_p, sku,
                                           cpu, ram, storage, gpu, display, os, battery, others,
                                           central_vid, bg_vid, inact_vid_1, inact_vid_2, inact_vid_3, timeout_str| {
        if let Some(ui) = save_admin_handle.upgrade() {
            let mut cfg = load_config();
            if !brand.trim().is_empty() { cfg.brand_override = Some(brand.to_string()); } else { cfg.brand_override = None; }
            if !model.trim().is_empty() { cfg.model_override = Some(model.to_string()); } else { cfg.model_override = None; }
            cfg.store_theme = retail.to_string();
            cfg.price_card = card_p.to_string();
            cfg.price_all_methods = normal_p.to_string();
            cfg.sku = sku.to_string();

            if !cpu.trim().is_empty() { cfg.processor_override = Some(cpu.to_string()); } else { cfg.processor_override = None; }
            if !ram.trim().is_empty() { cfg.ram_override = Some(ram.to_string()); } else { cfg.ram_override = None; }
            if !storage.trim().is_empty() { cfg.storage_override = Some(storage.to_string()); } else { cfg.storage_override = None; }
            if !gpu.trim().is_empty() { cfg.gpu_override = Some(gpu.to_string()); } else { cfg.gpu_override = None; }
            if !display.trim().is_empty() { cfg.display_override = Some(display.to_string()); } else { cfg.display_override = None; }
            if !os.trim().is_empty() { cfg.os_override = Some(os.to_string()); } else { cfg.os_override = None; }
            if !battery.trim().is_empty() {
                let b = battery.trim();
                let formatted = if let Ok(num) = b.parse::<f32>() {
                    format!("Hasta {} horas", num)
                } else if b.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
                    format!("Hasta {} horas", b)
                } else if !b.to_lowercase().contains("hora") && !b.to_lowercase().contains("hrs") && !b.to_lowercase().contains("h") {
                    format!("Hasta {} horas", b)
                } else {
                    b.to_string()
                };
                cfg.battery_override = Some(formatted);
            } else {
                cfg.battery_override = None;
            }
            cfg.other_specs = others.to_string();

            if !central_vid.trim().is_empty() {
                if central_vid.to_string() != cfg.central_video_path {
                    let resolved = domains::media::resolve_video_path(&central_vid);
                    if let Ok(mut guard) = landing_player_on_save.lock() {
                        if let Some(ref mut p) = *guard {
                            p.change_video_looping(&resolved.to_string_lossy(), true);
                        }
                    }
                }
                cfg.central_video_path = central_vid.to_string();
            }
            if !bg_vid.trim().is_empty() {
                cfg.bg_video_path = bg_vid.to_string();
            }
            if !inact_vid_1.trim().is_empty() { cfg.inactivity_video_path = inact_vid_1.to_string(); }
            cfg.inactivity_video_path_2 = inact_vid_2.to_string();
            cfg.inactivity_video_path_3 = inact_vid_3.to_string();
            if let Ok(t) = timeout_str.trim().parse::<u64>() {
                if t > 0 { cfg.inactivity_timeout_sec = t; }
            }

            let _ = save_config(&cfg);
            ui.set_is_admin_open(false);
            ui.set_is_admin_authenticated(false);
            populate_ui_state(&ui);
        }
    });

    // Vault: Import video to safe folder
    let vault_handle = main_window.as_weak();
    main_window.on_import_video_to_vault_clicked(move || {
        let wh = vault_handle.clone();
        std::thread::spawn(move || {
            if let Some(picked) = domains::media::vault::pick_video_file() {
                if let Ok(_item) = domains::media::vault::import_video_to_vault(&picked) {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = wh.upgrade() {
                            populate_vault_videos(&ui);
                        }
                    });
                }
            }
        });
    });

    // Vault: Delete video
    let del_vault_handle = main_window.as_weak();
    main_window.on_delete_vault_video_clicked(move |filename| {
        let _ = domains::media::vault::delete_vault_video(&filename);
        if let Some(ui) = del_vault_handle.upgrade() {
            populate_vault_videos(&ui);
        }
    });

    // Helper for picking video files and setting fields
    let spawn_video_picker = |ui_weak: slint::Weak<MainWindow>, setter: Box<dyn Fn(&MainWindow, String) + Send + 'static>| {
        std::thread::spawn(move || {
            if let Some(picked) = domains::media::vault::pick_video_file() {
                let chosen_path = if let Ok(item) = domains::media::vault::import_video_to_vault(&picked) {
                    item.path
                } else {
                    picked
                };
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        setter(&ui, chosen_path);
                        populate_vault_videos(&ui);
                    }
                });
            }
        });
    };

    let h_central = main_window.as_weak();
    let picker_central = spawn_video_picker.clone();
    main_window.on_pick_central_video_clicked(move || {
        picker_central(h_central.clone(), Box::new(|ui, path| {
            ui.set_cfg_central_video(path.into());
        }));
    });

    let h_bg = main_window.as_weak();
    let picker_bg = spawn_video_picker.clone();
    main_window.on_pick_bg_video_clicked(move || {
        picker_bg(h_bg.clone(), Box::new(|ui, path| {
            ui.set_cfg_bg_video(path.into());
        }));
    });

    let h_inact1 = main_window.as_weak();
    let picker_inact1 = spawn_video_picker.clone();
    main_window.on_pick_inactivity_1_clicked(move || {
        picker_inact1(h_inact1.clone(), Box::new(|ui, path| {
            ui.set_cfg_inactivity_video_1(path.into());
        }));
    });

    let h_inact2 = main_window.as_weak();
    let picker_inact2 = spawn_video_picker.clone();
    main_window.on_pick_inactivity_2_clicked(move || {
        picker_inact2(h_inact2.clone(), Box::new(|ui, path| {
            ui.set_cfg_inactivity_video_2(path.into());
        }));
    });

    let h_inact3 = main_window.as_weak();
    let picker_inact3 = spawn_video_picker.clone();
    main_window.on_pick_inactivity_3_clicked(move || {
        picker_inact3(h_inact3.clone(), Box::new(|ui, path| {
            ui.set_cfg_inactivity_video_3(path.into());
        }));
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

    // Reset Store Defaults
    let reset_store_handle = main_window.as_weak();
    main_window.on_reset_store_defaults(move || {
        if let Some(ui) = reset_store_handle.upgrade() {
            let hw = detect_hardware();
            let brand = if !hw.brand.is_empty() && !hw.brand.to_lowercase().contains("to be filled") && !hw.brand.to_lowercase().contains("o.e.m.") {
                hw.brand
            } else {
                "ASUS".to_string()
            };
            let model = if !hw.model.is_empty() && !hw.model.to_lowercase().contains("to be filled") && !hw.model.to_lowercase().contains("o.e.m.") {
                hw.model
            } else {
                "ROG Strix PC".to_string()
            };
            ui.set_cfg_brand(brand.into());
            ui.set_cfg_model(model.into());
            ui.set_cfg_retail("ripley".into());
            ui.set_cfg_card_price("$1.299.990".into());
            ui.set_cfg_normal_price("$1.349.990".into());
            ui.set_cfg_sku("8493021".into());
        }
    });

    // Reset Specs Defaults
    let reset_specs_handle = main_window.as_weak();
    main_window.on_reset_specs_defaults(move || {
        if let Some(ui) = reset_specs_handle.upgrade() {
            let hw = detect_hardware();
            let brand = if !hw.brand.is_empty() && !hw.brand.to_lowercase().contains("to be filled") && !hw.brand.to_lowercase().contains("o.e.m.") {
                hw.brand
            } else {
                "ASUS".to_string()
            };
            let model = if !hw.model.is_empty() && !hw.model.to_lowercase().contains("to be filled") && !hw.model.to_lowercase().contains("o.e.m.") {
                hw.model
            } else {
                "ROG Strix PC".to_string()
            };
            ui.set_cfg_brand(brand.into());
            ui.set_cfg_model(model.into());
            ui.set_cfg_cpu(hw.processor.into());
            ui.set_cfg_ram(hw.ram.into());
            ui.set_cfg_storage(hw.storage.into());
            ui.set_cfg_gpu(hw.gpu.into());
            ui.set_cfg_display(hw.display.into());
            ui.set_cfg_os(hw.os.into());
            ui.set_cfg_battery(if !hw.battery.is_empty() && !hw.battery.contains("Conectado AC") {
                hw.battery.into()
            } else {
                "Hasta 8 horas".into()
            });
            ui.set_cfg_others("Wi-Fi 6 / Bluetooth 5.3 / Teclado en Español".into());
        }
    });

    // Reset Videos Defaults
    let reset_videos_handle = main_window.as_weak();
    main_window.on_reset_videos_defaults(move || {
        if let Some(ui) = reset_videos_handle.upgrade() {
            ui.set_cfg_central_video("assets/videos/landing.mp4".into());
            ui.set_cfg_bg_video("assets/videos/bg.mp4".into());
            ui.set_cfg_inactivity_video_1("assets/videos/inactivity.mp4".into());
            ui.set_cfg_inactivity_video_2("".into());
            ui.set_cfg_inactivity_video_3("".into());
            ui.set_cfg_inactivity_timeout("120".into());
        }
    });

    // Reset System / Factory Defaults
    let reset_sys_handle = main_window.as_weak();
    main_window.on_reset_system_defaults(move || {
        if let Some(ui) = reset_sys_handle.upgrade() {
            let default_cfg = domains::showcase::config::AppConfig::default();
            let _ = domains::showcase::config::save_config(&default_cfg);
            populate_ui_state(&ui);
        }
    });

    // Floating "Ver Detalles" Button Widget
    let floating_btn = Rc::new(FloatingDetailsButton::new().expect("Failed to initialize FloatingDetailsButton"));
    let restore_mw_handle = main_window.as_weak();
    let restore_fb_handle = floating_btn.as_weak();
    let player_restore = player_shared.clone();

    floating_btn.on_restore_clicked(move || {
        IS_IN_TRY_PC_MODE.store(false, Ordering::SeqCst);
        let landing_video = domains::media::resolve_showroom_video_path();
        if let Ok(mut guard) = player_restore.lock() {
            if let Some(ref mut p) = *guard {
                p.change_video_looping(&landing_video.to_string_lossy(), true);
            }
        }

        let mwh = restore_mw_handle.clone();
        let fbh = restore_fb_handle.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(mw) = mwh.upgrade() {
                let _ = mw.show();
                mw.window().set_fullscreen(true);
                mw.set_is_screensaver_active(false);
            }

            #[cfg(target_os = "windows")]
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetSystemMetrics, SetForegroundWindow, SetWindowPos, ShowWindow, HWND_TOP,
                    SM_CXSCREEN, SM_CYSCREEN, SWP_SHOWWINDOW, SW_RESTORE, SW_SHOW,
                };
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

            if let Some(fb) = fbh.upgrade() {
                let _ = fb.hide();
            }
        });
    });

    // Try PC Mode
    let try_pc_fb_handle = floating_btn.as_weak();
    main_window.on_try_pc_clicked(move || {
        IS_IN_TRY_PC_MODE.store(true, Ordering::SeqCst);
        if let Some(fb) = try_pc_fb_handle.upgrade() {
            let _ = fb.show();
            #[cfg(target_os = "windows")]
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    FindWindowA, GetSystemMetrics, SetForegroundWindow, SetWindowPos, HWND_TOPMOST,
                    SM_CXSCREEN, SM_CYSCREEN, SWP_SHOWWINDOW,
                };
                let btn_title = b"Zenit - Ver Detalles\0";
                let btn_hwnd = FindWindowA(std::ptr::null(), btn_title.as_ptr());
                if !btn_hwnd.is_null() {
                    FLOATING_HWND.store(btn_hwnd as isize, Ordering::SeqCst);
                    let screen_w = GetSystemMetrics(SM_CXSCREEN);
                    let screen_h = GetSystemMetrics(SM_CYSCREEN);
                    let w = 220;
                    let h = 60;
                    let x = screen_w - w - 40;
                    let y = screen_h - h - 60;
                    SetWindowPos(btn_hwnd, HWND_TOPMOST, x, y, w, h, SWP_SHOWWINDOW);
                    SetForegroundWindow(btn_hwnd);
                }
            }
        }

        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_MINIMIZE};
            let main_hwnd = MAIN_HWND.load(Ordering::SeqCst) as HWND;
            if !main_hwnd.is_null() {
                ShowWindow(main_hwnd, SW_MINIMIZE);
                ShowWindow(main_hwnd, SW_HIDE);
            }
        }
    });

    // Screensaver Dismiss Tracking & Dismiss Handler
    let last_dismiss_time = std::sync::Arc::new(std::sync::Mutex::new(std::time::Instant::now()));
    let dismiss_handle = main_window.as_weak();
    let player_dismiss = player_shared.clone();
    let last_dismiss_for_cb = last_dismiss_time.clone();
    main_window.on_dismiss_screensaver(move || {
        if let Ok(mut t) = last_dismiss_for_cb.lock() {
            *t = std::time::Instant::now();
        }
        if let Some(ui) = dismiss_handle.upgrade() {
            ui.set_is_screensaver_active(false);
            IS_SCREENSAVER_ACTIVE.store(false, Ordering::SeqCst);
        }
        let landing_video = domains::media::resolve_showroom_video_path();
        if let Ok(mut guard) = player_dismiss.lock() {
            if let Some(ref mut p) = *guard {
                p.change_video_looping(&landing_video.to_string_lossy(), true);
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
    let last_dismiss_watcher = last_dismiss_time.clone();
    std::thread::spawn(move || {
        let mut inactivity_slot = 0usize;
        let mut screensaver_start_time = std::time::Instant::now();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let idle_seconds = get_system_idle_seconds();
            let cfg = load_config();
            let timeout = cfg.inactivity_timeout_sec;

            let time_since_dismiss = last_dismiss_watcher
                .lock()
                .map(|t| t.elapsed().as_secs())
                .unwrap_or(u64::MAX);

            let was_screensaver = IS_SCREENSAVER_ACTIVE.load(Ordering::SeqCst);
            let in_try_pc = IS_IN_TRY_PC_MODE.load(Ordering::SeqCst);

            // 1. In Try-PC Mode and Idle >= Timeout: Auto-restore Main Window AND trigger Inactivity Screensaver
            if in_try_pc && idle_seconds >= timeout {
                IS_IN_TRY_PC_MODE.store(false, Ordering::SeqCst);
                IS_SCREENSAVER_ACTIVE.store(true, Ordering::SeqCst);
                screensaver_start_time = std::time::Instant::now();

                let video_list = cfg.get_inactivity_video_list();
                if !video_list.is_empty() {
                    let selected_rel_path = &video_list[inactivity_slot % video_list.len()];
                    inactivity_slot = inactivity_slot.wrapping_add(1);
                    let promo_video = domains::media::resolve_video_path(selected_rel_path);
                    if promo_video.exists() {
                        if let Ok(mut guard) = player_watcher.lock() {
                            if let Some(ref mut p) = *guard {
                                p.change_video_looping(&promo_video.to_string_lossy(), false);
                            }
                        }
                    }
                }

                let wh = watcher_handle.clone();
                let fbh = watcher_fb_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(fb) = fbh.upgrade() {
                        let _ = fb.hide();
                    }
                    if let Some(ui) = wh.upgrade() {
                        let _ = ui.show();
                        ui.window().set_fullscreen(true);
                        ui.set_is_screensaver_active(true);
                        #[cfg(target_os = "windows")]
                        unsafe {
                            use windows_sys::Win32::UI::WindowsAndMessaging::{
                                GetSystemMetrics, SetForegroundWindow, SetWindowPos, ShowWindow,
                                HWND_TOP, SM_CXSCREEN, SM_CYSCREEN, SWP_SHOWWINDOW, SW_RESTORE,
                                SW_SHOW,
                            };
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
                });
                continue;
            }

            // 2. On Main Window (Not in Try-PC, Not in Screensaver): Trigger Inactivity Screensaver
            if !in_try_pc && !was_screensaver {
                if idle_seconds >= timeout && time_since_dismiss >= timeout {
                    let video_list = cfg.get_inactivity_video_list();
                    if !video_list.is_empty() {
                        let selected_rel_path = &video_list[inactivity_slot % video_list.len()];
                        inactivity_slot = inactivity_slot.wrapping_add(1);
                        let promo_video = domains::media::resolve_video_path(selected_rel_path);
                        if promo_video.exists() {
                            if let Ok(mut guard) = player_watcher.lock() {
                                if let Some(ref mut p) = *guard {
                                    p.change_video_looping(&promo_video.to_string_lossy(), false);
                                }
                            }
                        }
                    }

                    IS_SCREENSAVER_ACTIVE.store(true, Ordering::SeqCst);
                    screensaver_start_time = std::time::Instant::now();

                    let wh = watcher_handle.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = wh.upgrade() {
                            ui.set_is_screensaver_active(true);
                        }
                    });
                }
            } else if was_screensaver {
                // 3. Currently in Screensaver: Dismiss if user interacts OR if inactivity video finishes
                let mut video_finished = false;
                if screensaver_start_time.elapsed().as_secs() >= 2 {
                    if let Ok(guard) = player_watcher.lock() {
                        if let Some(ref p) = *guard {
                            video_finished = p.is_eof();
                            if !video_finished {
                                if let (Some(pos), Some(dur)) = (p.get_time_pos(), p.get_duration()) {
                                    if dur > 1.0 && pos >= dur - 0.5 {
                                        video_finished = true;
                                    }
                                }
                            }
                        }
                    }
                }

                if idle_seconds < 2 || video_finished {
                    IS_SCREENSAVER_ACTIVE.store(false, Ordering::SeqCst);
                    if let Ok(mut t) = last_dismiss_watcher.lock() {
                        *t = std::time::Instant::now();
                    }

                    let landing_video = domains::media::resolve_showroom_video_path();
                    if let Ok(mut guard) = player_watcher.lock() {
                        if let Some(ref mut p) = *guard {
                            p.change_video_looping(&landing_video.to_string_lossy(), true);
                        }
                    }

                    let wh = watcher_handle.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = wh.upgrade() {
                            ui.set_is_screensaver_active(false);
                        }
                    });
                }
            }
        }
    });

    // Ensure true fullscreen kiosk mode
    main_window.window().set_fullscreen(true);

    // Start native event loop
    main_window.run().expect("Error running Slint application");
}
