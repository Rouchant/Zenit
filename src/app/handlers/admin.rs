use crate::app::state::{get_last_dismiss_tracker, IS_SCREENSAVER_ACTIVE};
use crate::app::sync::populate_ui_state;
use crate::domains::hardware::detect_hardware;
use crate::domains::media::SlintMpvPlayer;
use crate::domains::showcase::{load_config, save_config, verify_password};
use crate::MainWindow;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn setup_admin_handlers(
    main_window: &MainWindow,
    player_shared: Arc<Mutex<Option<SlintMpvPlayer>>>,
) {
    // 1. 4-Tap Admin Trigger in top-right corner
    let tap_tracker = Rc::new(RefCell::new((0u32, Instant::now())));
    let tap_handle = main_window.as_weak();
    main_window.on_corner_tap_clicked(move || {
        let mut st = tap_tracker.borrow_mut();
        let now = Instant::now();
        if now.duration_since(st.1).as_millis() < 800 {
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

    // 2. Close Admin Panel
    let close_admin_handle = main_window.as_weak();
    main_window.on_close_admin_clicked(move || {
        if let Some(ui) = close_admin_handle.upgrade() {
            ui.set_is_admin_open(false);
            ui.set_is_admin_authenticated(false);
            ui.set_admin_error_msg("".into());
        }
    });

    // 3. Verify Admin Password
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

    // 4. Auto-detect Hardware for Quick Setup & Admin Panel
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

    // 5. Save Admin Configuration Form
    let save_admin_handle = main_window.as_weak();
    let landing_player_on_save = player_shared.clone();
    main_window.on_save_admin_config(
        move |brand,
              model,
              retail,
              card_p,
              normal_p,
              sku,
              cpu,
              ram,
              storage,
              gpu,
              display,
              os,
              battery,
              others,
              central_vid,
              bg_vid,
              inact_vid_1,
              inact_vid_2,
              inact_vid_3,
              timeout_str| {
            if let Some(ui) = save_admin_handle.upgrade() {
                let mut cfg = load_config();
                if !brand.trim().is_empty() {
                    cfg.brand_override = Some(brand.to_string());
                } else {
                    cfg.brand_override = None;
                }
                if !model.trim().is_empty() {
                    cfg.model_override = Some(model.to_string());
                } else {
                    cfg.model_override = None;
                }
                cfg.store_theme = retail.to_string();
                cfg.price_card = card_p.to_string();
                cfg.price_all_methods = normal_p.to_string();
                cfg.sku = sku.to_string();

                if !cpu.trim().is_empty() {
                    cfg.processor_override = Some(cpu.to_string());
                } else {
                    cfg.processor_override = None;
                }
                if !ram.trim().is_empty() {
                    cfg.ram_override = Some(ram.to_string());
                } else {
                    cfg.ram_override = None;
                }
                if !storage.trim().is_empty() {
                    cfg.storage_override = Some(storage.to_string());
                } else {
                    cfg.storage_override = None;
                }
                if !gpu.trim().is_empty() {
                    cfg.gpu_override = Some(gpu.to_string());
                } else {
                    cfg.gpu_override = None;
                }
                if !display.trim().is_empty() {
                    cfg.display_override = Some(display.to_string());
                } else {
                    cfg.display_override = None;
                }
                if !os.trim().is_empty() {
                    cfg.os_override = Some(os.to_string());
                } else {
                    cfg.os_override = None;
                }
                if !battery.trim().is_empty() {
                    let b = battery.trim();
                    let formatted = if let Ok(num) = b.parse::<f32>() {
                        if num >= 20.0 {
                            format!("Batería {} Wh", num.round() as u64)
                        } else {
                            format!("Hasta {} horas", num)
                        }
                    } else if b.to_lowercase().contains("wh") {
                        if b.to_lowercase().starts_with("bater") {
                            b.to_string()
                        } else {
                            format!("Batería {}", b)
                        }
                    } else if b.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
                        if let Ok(num) = b.replace(',', ".").parse::<f32>() {
                            if num >= 20.0 {
                                format!("Batería {} Wh", num.round() as u64)
                            } else {
                                format!("Hasta {} horas", num)
                            }
                        } else {
                            format!("Batería {}", b)
                        }
                    } else if !b.to_lowercase().contains("hora")
                        && !b.to_lowercase().contains("hrs")
                        && !b.to_lowercase().contains("h")
                        && !b.to_lowercase().contains("bater")
                    {
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
                        let resolved = crate::domains::media::resolve_video_path(&central_vid);
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
                if !inact_vid_1.trim().is_empty() {
                    cfg.inactivity_video_path = inact_vid_1.to_string();
                }
                cfg.inactivity_video_path_2 = inact_vid_2.to_string();
                cfg.inactivity_video_path_3 = inact_vid_3.to_string();
                if let Ok(t) = timeout_str.trim().parse::<u64>() {
                    if t > 0 {
                        cfg.inactivity_timeout_sec = t;
                    }
                }

                let _ = save_config(&cfg);
                ui.set_is_admin_open(false);
                ui.set_is_admin_authenticated(false);
                populate_ui_state(&ui);
            }
        },
    );

    // 6. Save Quick Setup Form
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

    // 7. Reset Store Defaults
    let reset_store_handle = main_window.as_weak();
    main_window.on_reset_store_defaults(move || {
        if let Some(ui) = reset_store_handle.upgrade() {
            let hw = detect_hardware();
            let brand = if !hw.brand.is_empty()
                && !hw.brand.to_lowercase().contains("to be filled")
                && !hw.brand.to_lowercase().contains("o.e.m.")
            {
                hw.brand
            } else {
                "ASUS".to_string()
            };
            let model = if !hw.model.is_empty()
                && !hw.model.to_lowercase().contains("to be filled")
                && !hw.model.to_lowercase().contains("o.e.m.")
            {
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

    // 8. Reset Specs Defaults
    let reset_specs_handle = main_window.as_weak();
    main_window.on_reset_specs_defaults(move || {
        if let Some(ui) = reset_specs_handle.upgrade() {
            let hw = detect_hardware();
            let brand = if !hw.brand.is_empty()
                && !hw.brand.to_lowercase().contains("to be filled")
                && !hw.brand.to_lowercase().contains("o.e.m.")
            {
                hw.brand
            } else {
                "ASUS".to_string()
            };
            let model = if !hw.model.is_empty()
                && !hw.model.to_lowercase().contains("to be filled")
                && !hw.model.to_lowercase().contains("o.e.m.")
            {
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

    // 9. Reset Videos Defaults
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

    // 10. Reset System Defaults
    let reset_sys_handle = main_window.as_weak();
    main_window.on_reset_system_defaults(move || {
        if let Some(ui) = reset_sys_handle.upgrade() {
            let default_cfg = crate::domains::showcase::config::AppConfig::default();
            let _ = crate::domains::showcase::config::save_config(&default_cfg);
            populate_ui_state(&ui);
        }
    });

    // 11. Screensaver Dismiss Callback
    let dismiss_handle = main_window.as_weak();
    let player_dismiss = player_shared.clone();
    let last_dismiss_tracker = get_last_dismiss_tracker();
    main_window.on_dismiss_screensaver(move || {
        if let Ok(mut t) = last_dismiss_tracker.lock() {
            *t = Instant::now();
        }
        if let Some(ui) = dismiss_handle.upgrade() {
            ui.set_is_screensaver_active(false);
            IS_SCREENSAVER_ACTIVE.store(false, Ordering::SeqCst);
        }
        let landing_video = crate::domains::media::resolve_showroom_video_path();
        if let Ok(mut guard) = player_dismiss.lock() {
            if let Some(ref mut p) = *guard {
                p.change_video_looping(&landing_video.to_string_lossy(), true);
            }
        }
    });

    // 12. Emergency Exit & Restart
    main_window.on_emergency_exit_clicked(|| {
        std::process::exit(0);
    });

    main_window.on_emergency_restart_clicked(|| {
        if let Ok(current_exe) = std::env::current_exe() {
            let _ = std::process::Command::new(current_exe).spawn();
        }
        std::process::exit(0);
    });
}
