use crate::app::sync::populate_ui_state;
use crate::domains::media::SlintMpvPlayer;
use crate::domains::showcase::{load_config, save_config};
use crate::MainWindow;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};

pub fn setup_save_handlers(
    main_window: &MainWindow,
    player_shared: Arc<Mutex<Option<SlintMpvPlayer>>>,
) {
    // 1. Save Admin Configuration Form
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
                cfg.brand_override = if !brand.trim().is_empty() {
                    Some(brand.to_string())
                } else {
                    None
                };
                cfg.model_override = if !model.trim().is_empty() {
                    Some(model.to_string())
                } else {
                    None
                };
                cfg.store_theme = retail.to_string();
                cfg.price_card = crate::app::sync::format_price_input(&card_p);
                cfg.price_all_methods = crate::app::sync::format_price_input(&normal_p);
                cfg.sku = sku.to_string();

                cfg.processor_override = if !cpu.trim().is_empty() {
                    Some(cpu.to_string())
                } else {
                    None
                };
                cfg.ram_override = if !ram.trim().is_empty() {
                    Some(ram.to_string())
                } else {
                    None
                };
                cfg.storage_override = if !storage.trim().is_empty() {
                    Some(storage.to_string())
                } else {
                    None
                };
                cfg.gpu_override = if !gpu.trim().is_empty() {
                    Some(gpu.to_string())
                } else {
                    None
                };
                cfg.display_override = if !display.trim().is_empty() {
                    Some(display.to_string())
                } else {
                    None
                };
                cfg.os_override = if !os.trim().is_empty() {
                    Some(os.to_string())
                } else {
                    None
                };

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

    // 2. Save Quick Setup Form
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
            cfg.price_card = crate::app::sync::format_price_input(&card_p);
            cfg.price_all_methods = crate::app::sync::format_price_input(&normal_p);
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
}
