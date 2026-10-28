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
                let logo_str = ui.get_cfg_brand_logo().to_string();
                cfg.brand_logo_override = if !logo_str.trim().is_empty() {
                    Some(logo_str.trim().to_string())
                } else {
                    None
                };
                cfg.model_override = if !model.trim().is_empty() {
                    Some(model.to_string())
                } else {
                    None
                };
                cfg.store_theme = retail.to_string();
                let store_logo_str = ui.get_cfg_store_logo().to_string();
                cfg.store_logo_override = if !store_logo_str.trim().is_empty() {
                    Some(store_logo_str.trim().to_string())
                } else {
                    None
                };
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
                        format!("Batería {} Wh", num.round() as u64)
                    } else if b.to_lowercase().contains("wh") {
                        if b.to_lowercase().starts_with("bater") {
                            b.to_string()
                        } else {
                            format!("Batería {}", b)
                        }
                    } else if b.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
                        if let Ok(num) = b.replace(',', ".").parse::<f32>() {
                            format!("Batería {} Wh", num.round() as u64)
                        } else {
                            format!("Batería {} Wh", b)
                        }
                    } else {
                        b.to_string()
                    };
                    cfg.battery_override = Some(formatted);
                } else {
                    cfg.battery_override = None;
                }
                cfg.other_specs = others.to_string();

                let parse_badge = |s: slint::SharedString| {
                    let trimmed = s.trim();
                    Some(trimmed.to_string())
                };
                cfg.badge_cpu = parse_badge(ui.get_cfg_badge_cpu());
                cfg.badge_ram = parse_badge(ui.get_cfg_badge_ram());
                cfg.badge_storage = parse_badge(ui.get_cfg_badge_storage());
                cfg.badge_gpu = parse_badge(ui.get_cfg_badge_gpu());
                cfg.badge_display = parse_badge(ui.get_cfg_badge_display());
                cfg.badge_os = parse_badge(ui.get_cfg_badge_os());
                cfg.badge_battery = parse_badge(ui.get_cfg_badge_battery());
                cfg.badge_others = parse_badge(ui.get_cfg_badge_others());

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

                // Save UI Theme Colors
                let bg_base = ui.get_cfg_bg_base().to_string();
                let bg_start = ui.get_cfg_bg_start().to_string();
                let bg_mid = ui.get_cfg_bg_mid().to_string();
                let bg_end = ui.get_cfg_bg_end().to_string();
                let card_bg = ui.get_cfg_card_bg().to_string();
                let card_border = ui.get_cfg_card_border().to_string();
                let text_primary = ui.get_cfg_text_primary().to_string();
                let text_secondary = ui.get_cfg_text_secondary().to_string();
                let accent = ui.get_cfg_accent().to_string();
                let btn_bg = ui.get_cfg_btn_bg().to_string();
                let btn_text = ui.get_cfg_btn_text().to_string();

                if !bg_base.trim().is_empty() {
                    cfg.bg_base_color = bg_base;
                }
                if !bg_start.trim().is_empty() {
                    cfg.bg_start_color = bg_start;
                }
                if !bg_mid.trim().is_empty() {
                    cfg.bg_mid_color = bg_mid;
                }
                if !bg_end.trim().is_empty() {
                    cfg.bg_end_color = bg_end;
                }
                if !card_bg.trim().is_empty() {
                    cfg.card_bg_color = card_bg;
                }
                if !card_border.trim().is_empty() {
                    cfg.card_border_color = card_border;
                }
                if !text_primary.trim().is_empty() {
                    cfg.text_primary_color = text_primary;
                }
                if !text_secondary.trim().is_empty() {
                    cfg.text_secondary_color = text_secondary;
                }
                if !accent.trim().is_empty() {
                    cfg.accent_color = accent.clone();
                    cfg.primary_color = accent;
                }
                if !btn_bg.trim().is_empty() {
                    cfg.btn_bg_color = btn_bg;
                }
                if !btn_text.trim().is_empty() {
                    cfg.btn_text_color = btn_text;
                }
                cfg.is_dark_mode = ui.get_is_dark_mode();
                cfg.setup_completed = true;

                let _ = save_config(&cfg);
                ui.set_is_admin_open(false);
                ui.set_is_admin_authenticated(false);
                populate_ui_state(&ui);

                ui.set_toast_message("Cambios aplicados y guardados con éxito".into());
                ui.set_show_toast(true);
                let toast_handle = ui.as_weak();
                slint::Timer::single_shot(std::time::Duration::from_millis(2500), move || {
                    if let Some(w) = toast_handle.upgrade() {
                        w.set_show_toast(false);
                    }
                });
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
            let ret_trimmed = retail.trim();
            if !ret_trimmed.is_empty() {
                let ret_low = ret_trimmed.to_lowercase();
                cfg.store_theme = ret_low.clone();
                if ret_low == "falabella" {
                    cfg.store_logo_override = Some("assets/logos/falabella.svg".to_string());
                } else if ret_low == "ripley" {
                    cfg.store_logo_override = Some("assets/logos/ripley.svg".to_string());
                } else if ret_low == "paris" {
                    cfg.store_logo_override = Some("assets/logos/paris.svg".to_string());
                } else {
                    cfg.store_logo_override = None;
                }
            }
            cfg.setup_completed = true;
            let _ = save_config(&cfg);

            ui.set_is_quick_setup_open(false);
            populate_ui_state(&ui);
        }
    });
}
