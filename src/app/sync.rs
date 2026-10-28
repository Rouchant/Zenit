use crate::domains::hardware::{get_hardware_specs, is_oem_placeholder};
use crate::domains::showcase::{build_ui_state, load_config};
use slint::{ComponentHandle, ModelRc, VecModel};
use crate::{FloatingDetailsButton, MainWindow, SpecItem, VaultVideoItem};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

static FLOATING_BTN_HANDLE: Mutex<Option<slint::Weak<FloatingDetailsButton>>> = Mutex::new(None);

pub fn register_floating_button(btn: &FloatingDetailsButton) {
    if let Ok(mut guard) = FLOATING_BTN_HANDLE.lock() {
        *guard = Some(btn.as_weak());
    }
}

thread_local! {
    static ICON_CACHE: RefCell<HashMap<String, slint::Image>> = RefCell::new(HashMap::new());
}

fn load_cached_icon(path: &str) -> slint::Image {
    ICON_CACHE.with(|cache| {
        let mut map = cache.borrow_mut();
        if let Some(img) = map.get(path) {
            return img.clone();
        }
        let img = slint::Image::load_from_path(std::path::Path::new(path)).unwrap_or_default();
        map.insert(path.to_string(), img.clone());
        img
    })
}

pub fn populate_vault_videos(window: &MainWindow) {
    crate::domains::media::vault::seed_vault_defaults();
    let videos = crate::domains::media::vault::list_vault_videos();
    let items: Vec<VaultVideoItem> = videos
        .into_iter()
        .map(|v| {
            let is_preloaded = crate::domains::media::vault::is_preloaded_video(&v.filename);
            VaultVideoItem {
                name: v.name.into(),
                filename: v.filename.into(),
                path: v.path.into(),
                size_str: format!("{:.1} MB", v.size_mb).into(),
                is_preloaded,
            }
        })
        .collect();
    window.set_vault_videos(ModelRc::new(VecModel::from(items)));
}

pub fn format_price_input(input: &str) -> String {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).take(11).collect();
    if digits.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let chars: Vec<char> = digits.chars().collect();
    let len = chars.len();

    for (i, ch) in chars.into_iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }

    format!("$ {}", result)
}

pub fn format_clp_currency(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let has_digits = trimmed.chars().any(|c| c.is_ascii_digit());
    if !has_digits {
        return trimmed.to_string();
    }

    format_price_input(trimmed)
}

pub fn populate_ui_state(window: &MainWindow) {
    let state = build_ui_state();
    let cfg = load_config();

    window.set_brand_name(state.brand_name.clone().into());
    let model_display = cfg.model_override.clone().unwrap_or_else(|| {
        if !state.model_name.is_empty() {
            state.model_name.clone()
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

    let card_label = match store_trim.to_lowercase().as_str() {
        "falabella" => "CMR / Débito Falabella".to_string(),
        "ripley" => "Tarjeta Banco Ripley".to_string(),
        "paris" => "Tarjeta Cencosud".to_string(),
        _ if !retail_display.is_empty() => format!("Tarjeta {}", retail_display),
        _ => "Precio Tarjeta".to_string(),
    };
    window.set_card_label(card_label.into());
    window.set_card_price(format_clp_currency(card_trim).into());

    let normal_label = if card_trim.is_empty() {
        "Todo medio de pago".to_string()
    } else {
        "Otros medios de pago".to_string()
    };
    window.set_normal_label(normal_label.into());
    window.set_normal_price(format_clp_currency(normal_trim).into());

    window.set_sku_code(sku_trim.into());

    // Store Logo
    if let Some(ref logo_path) = state.store_logo {
        window.set_store_logo(load_cached_icon(logo_path));
    } else {
        window.set_store_logo(slint::Image::default());
    }

    // Brand Logo
    if let Some(ref logo_path) = state.brand_logo {
        window.set_brand_logo(load_cached_icon(logo_path));
        window.set_has_brand_logo(true);
    } else {
        window.set_brand_logo(slint::Image::default());
        window.set_has_brand_logo(false);
    }

    // Brand Logo Dark (#F5F5F5)
    if let Some(ref logo_dark_path) = state.brand_logo_dark {
        window.set_brand_logo_dark(load_cached_icon(logo_dark_path));
        window.set_has_brand_logo_dark(true);
    } else {
        window.set_brand_logo_dark(slint::Image::default());
        window.set_has_brand_logo_dark(false);
    }

    // Populate Left Column Specs
    let left_items: Vec<SpecItem> = state
        .left_specs
        .iter()
        .map(|s| {
            let icon_img = load_cached_icon(&s.icon);
            SpecItem {
                category: s.label.to_uppercase().into(),
                value: s.value.clone().into(),
                subvalue: s.sub_value.clone().unwrap_or_default().into(),
                tag: s.tag.clone().unwrap_or_default().into(),
                icon: icon_img,
            }
        })
        .collect();
    window.set_left_specs(ModelRc::new(VecModel::from(left_items)));

    // Populate Right Column Specs
    let right_items: Vec<SpecItem> = state
        .right_specs
        .iter()
        .map(|s| {
            let icon_img = load_cached_icon(&s.icon);
            SpecItem {
                category: s.label.to_uppercase().into(),
                value: s.value.clone().into(),
                subvalue: s.sub_value.clone().unwrap_or_default().into(),
                tag: s.tag.clone().unwrap_or_default().into(),
                icon: icon_img,
            }
        })
        .collect();
    window.set_right_specs(ModelRc::new(VecModel::from(right_items)));

    // Pre-populate Quick Setup & Admin Form Fields
    let hw = get_hardware_specs();

    let clean_brand = cfg.brand_override.as_deref()
        .filter(|b| !is_oem_placeholder(b))
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            if !hw.brand.is_empty() && !is_oem_placeholder(&hw.brand) {
                hw.brand.clone()
            } else {
                "ASUS".to_string()
            }
        });

    let clean_model = cfg.model_override.as_deref()
        .filter(|m| !is_oem_placeholder(m))
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            if !hw.model.is_empty() && !is_oem_placeholder(&hw.model) {
                hw.model.clone()
            } else {
                "ROG Strix PC".to_string()
            }
        });

    window.set_setup_brand(clean_brand.clone().into());
    window.set_setup_model(clean_model.clone().into());

    window.set_setup_retail(if !cfg.store_theme.is_empty() && !cfg.store_theme.eq_ignore_ascii_case("default") {
        cfg.store_theme.clone().into()
    } else {
        "".into()
    });
    window.set_setup_card_price(card_trim.into());
    window.set_setup_normal_price(normal_trim.into());
    window.set_setup_sku(sku_trim.into());

    // Pre-populate Admin Tienda
    window.set_cfg_brand(clean_brand.into());
    let brand_logo_val = cfg.brand_logo_override.clone().unwrap_or_else(|| {
        state.brand_logo.clone().unwrap_or_default()
    });
    window.set_cfg_brand_logo(brand_logo_val.into());
    window.set_cfg_model(clean_model.into());

    window.set_cfg_retail(if !cfg.store_theme.is_empty() && !cfg.store_theme.eq_ignore_ascii_case("default") {
        cfg.store_theme.clone().into()
    } else {
        "".into()
    });
    let store_logo_val = cfg.store_logo_override.clone().unwrap_or_else(|| {
        state.store_logo.clone().unwrap_or_default()
    });
    window.set_cfg_store_logo(store_logo_val.into());
    window.set_cfg_card_price(card_trim.into());
    window.set_cfg_normal_price(normal_trim.into());
    window.set_cfg_sku(sku_trim.into());

    // Pre-populate Admin Especificaciones
    let cpu_val = cfg.processor_override.clone().unwrap_or_else(|| {
        state.left_specs.iter()
            .find(|s| s.id == "card-processor")
            .map(|s| s.value.clone())
            .unwrap_or_else(|| hw.processor.clone())
    });
    window.set_cfg_cpu(cpu_val.into());

    let ram_val = cfg.ram_override.clone().unwrap_or_else(|| {
        state.left_specs.iter()
            .find(|s| s.id == "card-ram")
            .map(|s| s.value.clone())
            .unwrap_or_else(|| hw.ram.clone())
    });
    window.set_cfg_ram(ram_val.into());

    let storage_val = cfg.storage_override.clone().unwrap_or_else(|| {
        state.left_specs.iter()
            .find(|s| s.id == "card-storage")
            .map(|s| s.value.clone())
            .unwrap_or_else(|| hw.storage.clone())
    });
    window.set_cfg_storage(storage_val.into());

    let gpu_val = cfg.gpu_override.clone().unwrap_or_else(|| {
        state.left_specs.iter()
            .find(|s| s.id == "card-gpu")
            .map(|s| s.value.clone())
            .unwrap_or_else(|| hw.gpu.clone())
    });
    window.set_cfg_gpu(gpu_val.into());

    let display_val = cfg.display_override.clone().unwrap_or_else(|| {
        state.right_specs.iter()
            .find(|s| s.id == "card-display")
            .map(|s| s.value.clone())
            .unwrap_or_else(|| hw.display.clone())
    });
    window.set_cfg_display(display_val.into());

    let os_val = cfg.os_override.clone().unwrap_or_else(|| {
        state.right_specs.iter()
            .find(|s| s.id == "card-os")
            .map(|s| s.value.clone())
            .unwrap_or_else(|| hw.os.clone())
    });
    window.set_cfg_os(os_val.into());

    let battery_val = cfg.battery_override.clone().filter(|b| !b.trim().is_empty() && !b.eq_ignore_ascii_case("Hasta 8 horas")).unwrap_or_else(|| {
        let b = state.right_specs.iter()
            .find(|s| s.id == "card-battery")
            .map(|s| s.value.clone())
            .unwrap_or_else(|| hw.battery.clone());
        if !b.is_empty() && !b.contains("Conectado AC") {
            b
        } else {
            "Batería 50 Wh".to_string()
        }
    });
    window.set_cfg_battery(battery_val.into());

    let others_val = if cfg.other_specs.trim().is_empty()
        || cfg.other_specs == "Wi-Fi 6 / Bluetooth 5.3 / Teclado en Español"
    {
        if !hw.others.is_empty() {
            hw.others.clone()
        } else {
            cfg.other_specs.clone()
        }
    } else {
        cfg.other_specs.clone()
    };
    window.set_cfg_others(others_val.into());

    // Populate active badges from the spec cards (or hardware detection)
    let cur_badge_cpu = state
        .left_specs
        .iter()
        .find(|s| s.id == "card-processor")
        .and_then(|s| s.tag.clone())
        .filter(|t| !t.trim().is_empty())
        .or_else(|| {
            if !hw.generation.is_empty() {
                Some(hw.generation.clone())
            } else {
                Some("GEN".to_string())
            }
        })
        .unwrap_or_else(|| "GEN".to_string());

    let cur_badge_ram = state
        .left_specs
        .iter()
        .find(|s| s.id == "card-ram")
        .and_then(|s| s.tag.clone())
        .filter(|t| !t.trim().is_empty())
        .or_else(|| {
            if !hw.ram_type.is_empty() {
                Some(hw.ram_type.clone())
            } else {
                Some("RAM".to_string())
            }
        })
        .unwrap_or_else(|| "RAM".to_string());

    let cur_badge_storage = state
        .left_specs
        .iter()
        .find(|s| s.id == "card-storage")
        .and_then(|s| s.tag.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| {
            let low = hw.storage.to_lowercase();
            if low.contains("nvme") {
                "NVMe".to_string()
            } else {
                "SSD".to_string()
            }
        });

    let cur_badge_os = state
        .right_specs
        .iter()
        .find(|s| s.id == "card-os")
        .and_then(|s| s.tag.clone())
        .filter(|t| !t.trim().is_empty())
        .or_else(|| crate::domains::hardware::detect_os_badge(&hw.os))
        .unwrap_or_default();

    let cur_badge_display = state
        .right_specs
        .iter()
        .find(|s| s.id == "card-display")
        .and_then(|s| s.tag.clone())
        .filter(|t| !t.trim().is_empty())
        .or_else(|| {
            if hw.refresh_rate > 0 {
                Some(format!("{}Hz", hw.refresh_rate))
            } else {
                Some("165Hz".to_string())
            }
        })
        .unwrap_or_else(|| "FHD".to_string());

    let cur_badge_gpu = state
        .left_specs
        .iter()
        .find(|s| s.id == "card-gpu")
        .and_then(|s| s.tag.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| {
            let low = hw.gpu.to_lowercase();
            if low.contains("rtx") {
                "RTX".to_string()
            } else if low.contains("gtx") {
                "GTX".to_string()
            } else if low.contains("radeon") || low.contains("rx") {
                "Radeon".to_string()
            } else if low.contains("arc") {
                "Arc".to_string()
            } else {
                "GPU".to_string()
            }
        });

    let cur_badge_battery = state
        .right_specs
        .iter()
        .find(|s| s.id == "card-battery")
        .and_then(|s| s.tag.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_default();

    let cur_badge_others = state
        .right_specs
        .iter()
        .find(|s| s.id == "card-others" || s.id == "card-features")
        .and_then(|s| s.tag.clone())
        .unwrap_or_default();

    // Admin Spec Badges: Muestran directamente el valor activo en la interfaz
    let badge_cpu_val = match &cfg.badge_cpu {
        Some(custom) => custom.clone(),
        None => cur_badge_cpu.clone(),
    };
    let badge_ram_val = match &cfg.badge_ram {
        Some(custom) => custom.clone(),
        None => cur_badge_ram.clone(),
    };
    let badge_storage_val = match &cfg.badge_storage {
        Some(custom) => custom.clone(),
        None => cur_badge_storage.clone(),
    };
    let badge_gpu_val = match &cfg.badge_gpu {
        Some(custom) => custom.clone(),
        None => cur_badge_gpu.clone(),
    };
    let badge_display_val = match &cfg.badge_display {
        Some(custom) => custom.clone(),
        None => cur_badge_display.clone(),
    };
    let badge_os_val = match &cfg.badge_os {
        Some(custom) => custom.clone(),
        None => cur_badge_os.clone(),
    };
    let badge_battery_val = match &cfg.badge_battery {
        Some(custom) => custom.clone(),
        None => cur_badge_battery.clone(),
    };
    let badge_others_val = match &cfg.badge_others {
        Some(custom) => custom.clone(),
        None => cur_badge_others.clone(),
    };

    window.set_cfg_badge_cpu(badge_cpu_val.into());
    window.set_cfg_badge_ram(badge_ram_val.into());
    window.set_cfg_badge_storage(badge_storage_val.into());
    window.set_cfg_badge_gpu(badge_gpu_val.into());
    window.set_cfg_badge_display(badge_display_val.into());
    window.set_cfg_badge_os(badge_os_val.into());
    window.set_cfg_badge_battery(badge_battery_val.into());
    window.set_cfg_badge_others(badge_others_val.into());

    window.set_placeholder_badge_cpu("".into());
    window.set_placeholder_badge_ram("".into());
    window.set_placeholder_badge_storage("".into());
    window.set_placeholder_badge_gpu("".into());
    window.set_placeholder_badge_display("".into());
    window.set_placeholder_badge_os("".into());
    window.set_placeholder_badge_battery("".into());
    window.set_placeholder_badge_others("".into());

    // Admin Videos
    window.set_cfg_central_video(cfg.central_video_path.clone().into());
    window.set_cfg_bg_video(cfg.bg_video_path.clone().into());
    window.set_cfg_inactivity_video_1(cfg.inactivity_video_path.clone().into());
    window.set_cfg_inactivity_video_2(cfg.inactivity_video_path_2.clone().into());
    window.set_cfg_inactivity_video_3(cfg.inactivity_video_path_3.clone().into());
    window.set_cfg_inactivity_timeout(cfg.inactivity_timeout_sec.to_string().into());

    // Theme Colors Synchronization - Analogous 3-color background gradient
    let bg_base = if !cfg.bg_base_color.trim().is_empty() {
        cfg.bg_base_color.clone()
    } else {
        "#cce5f8".to_string()
    };
    let (c1, c2, c3) = generate_analogous_gradient(&bg_base);
    let bg_start_str = if !cfg.bg_start_color.trim().is_empty() {
        cfg.bg_start_color.clone()
    } else {
        c1
    };
    let bg_mid_str = if !cfg.bg_mid_color.trim().is_empty() {
        cfg.bg_mid_color.clone()
    } else {
        c2
    };
    let bg_end_str = if !cfg.bg_end_color.trim().is_empty() {
        cfg.bg_end_color.clone()
    } else {
        c3
    };

    let bg_start = parse_hex_color(&bg_start_str, slint::Color::from_rgb_u8(196, 233, 248));
    let bg_mid = parse_hex_color(&bg_mid_str, slint::Color::from_rgb_u8(204, 229, 248));
    let bg_end = parse_hex_color(&bg_end_str, slint::Color::from_rgb_u8(220, 226, 248));
    let card_bg = parse_hex_color(&cfg.card_bg_color, slint::Color::from_rgb_u8(245, 245, 245));
    let card_border = parse_hex_color(&cfg.card_border_color, slint::Color::from_rgb_u8(186, 230, 253));
    let text_primary = parse_hex_color(&cfg.text_primary_color, slint::Color::from_rgb_u8(15, 23, 42));
    let text_secondary = parse_hex_color(&cfg.text_secondary_color, slint::Color::from_rgb_u8(71, 85, 105));
    let accent = parse_hex_color(&cfg.accent_color, slint::Color::from_rgb_u8(2, 132, 199));
    let btn_bg = parse_hex_color(&cfg.btn_bg_color, slint::Color::from_rgb_u8(2, 132, 199));
    let btn_text = parse_hex_color(&cfg.btn_text_color, slint::Color::from_rgb_u8(245, 245, 245));

    window.set_theme_bg_start(bg_start);
    window.set_theme_bg_mid(bg_mid);
    window.set_theme_bg_end(bg_end);
    window.set_theme_card_bg(card_bg);
    window.set_theme_card_border(card_border);
    window.set_theme_text_primary(text_primary);
    window.set_theme_text_secondary(text_secondary);
    window.set_theme_accent(accent);
    window.set_theme_btn_bg(btn_bg);
    window.set_theme_btn_text(btn_text);

    // Sync floating details button theme if active
    if let Ok(guard) = FLOATING_BTN_HANDLE.lock() {
        if let Some(ref weak) = *guard {
            if let Some(fb) = weak.upgrade() {
                sync_floating_button_theme(&fb, &cfg);
            }
        }
    }

    // Admin Interfaz Strings
    window.set_cfg_bg_base(bg_base.into());
    window.set_cfg_bg_start(bg_start_str.into());
    window.set_cfg_bg_mid(bg_mid_str.into());
    window.set_cfg_bg_end(bg_end_str.into());
    window.set_cfg_card_bg(cfg.card_bg_color.clone().into());
    window.set_cfg_card_border(cfg.card_border_color.clone().into());
    window.set_cfg_text_primary(cfg.text_primary_color.clone().into());
    window.set_cfg_text_secondary(cfg.text_secondary_color.clone().into());
    window.set_cfg_accent(cfg.accent_color.clone().into());
    window.set_cfg_btn_bg(cfg.btn_bg_color.clone().into());
    window.set_cfg_btn_text(cfg.btn_text_color.clone().into());
    window.set_is_dark_mode(cfg.is_dark_mode);

    let is_dark_bg = cfg.is_dark_mode || is_color_dark(bg_start);
    window.set_is_dark_bg(is_dark_bg);

    populate_vault_videos(window);

    // Primer Arranque: Si la configuración inicial no ha sido completada, abrir Quick Setup
    if !cfg.setup_completed {
        window.set_is_quick_setup_open(true);
    }
}

pub fn is_color_dark(color: slint::Color) -> bool {
    let r = color.red() as f32;
    let g = color.green() as f32;
    let b = color.blue() as f32;
    (0.299 * r + 0.587 * g + 0.114 * b) < 140.0
}

pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return Some((r, g, b));
        }
    }
    None
}

pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r_f = r as f32 / 255.0;
    let g_f = g as f32 / 255.0;
    let b_f = b as f32 / 255.0;

    let max = r_f.max(g_f).max(b_f);
    let min = r_f.min(g_f).min(b_f);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-5 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r_f).abs() < 1e-5 {
        ((g_f - b_f) / d + if g_f < b_f { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g_f).abs() < 1e-5 {
        ((b_f - r_f) / d + 2.0) / 6.0
    } else {
        ((r_f - g_f) / d + 4.0) / 6.0
    } * 360.0;

    (h, s, l)
}

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let h_norm = ((h % 360.0) + 360.0) % 360.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h_norm / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

    let (r1, g1, b1) = if (0.0..1.0).contains(&h_prime) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h_prime) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h_prime) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h_prime) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h_prime) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let m = l - c / 2.0;
    // Cap at 245 (#F5) per visual requirement
    let r = (((r1 + m) * 255.0).round().clamp(0.0, 245.0)) as u8;
    let g = (((g1 + m) * 255.0).round().clamp(0.0, 245.0)) as u8;
    let b = (((b1 + m) * 255.0).round().clamp(0.0, 245.0)) as u8;

    (r, g, b)
}

pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r.min(0xf5), g.min(0xf5), b.min(0xf5))
}

pub fn generate_analogous_gradient(base_hex: &str) -> (String, String, String) {
    let (r, g, b) = hex_to_rgb(base_hex).unwrap_or((227, 232, 245));
    let (h, s, l) = rgb_to_hsl(r, g, b);

    if s < 0.05 {
        // Monochromatic / grayscale base
        let l1 = (l - 0.03).clamp(0.0, 0.96);
        let l2 = l.clamp(0.0, 0.96);
        let l3 = (l + 0.03).clamp(0.0, 0.96);
        let (r1, g1, b1) = hsl_to_rgb(0.0, 0.0, l1);
        let (r2, g2, b2) = hsl_to_rgb(0.0, 0.0, l2);
        let (r3, g3, b3) = hsl_to_rgb(0.0, 0.0, l3);
        (rgb_to_hex(r1, g1, b1), rgb_to_hex(r2, g2, b2), rgb_to_hex(r3, g3, b3))
    } else {
        // Boost saturation slightly for light backgrounds to make them vibrant and rich
        let s_eff = if l > 0.65 {
            (s * 1.35).clamp(0.0, 1.0)
        } else {
            s
        };

        // Analogous 3-color palette: -25 deg, base, +25 deg
        let h1 = (h - 25.0 + 360.0) % 360.0;
        let h2 = h;
        let h3 = (h + 25.0) % 360.0;

        let (r1, g1, b1) = hsl_to_rgb(h1, s_eff, l);
        let (r2, g2, b2) = hsl_to_rgb(h2, s_eff, l);
        let (r3, g3, b3) = hsl_to_rgb(h3, s_eff, l);
        (rgb_to_hex(r1, g1, b1), rgb_to_hex(r2, g2, b2), rgb_to_hex(r3, g3, b3))
    }
}

pub fn parse_hex_color(hex: &str, fallback: slint::Color) -> slint::Color {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return slint::Color::from_rgb_u8(r, g, b);
        }
    } else if clean.len() == 8 {
        if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
            u8::from_str_radix(&clean[6..8], 16),
        ) {
            return slint::Color::from_argb_u8(a, r, g, b);
        }
    }
    fallback
}

pub fn sync_floating_button_theme(floating_btn: &FloatingDetailsButton, cfg: &crate::domains::showcase::AppConfig) {
    let card_bg = parse_hex_color(&cfg.card_bg_color, slint::Color::from_rgb_u8(245, 245, 245));
    let card_border = parse_hex_color(&cfg.card_border_color, slint::Color::from_rgb_u8(226, 232, 240));
    let text_primary = parse_hex_color(&cfg.text_primary_color, slint::Color::from_rgb_u8(15, 23, 42));
    let text_secondary = parse_hex_color(&cfg.text_secondary_color, slint::Color::from_rgb_u8(100, 116, 139));
    let accent = parse_hex_color(&cfg.accent_color, slint::Color::from_rgb_u8(2, 132, 199));
    let btn_bg = parse_hex_color(&cfg.btn_bg_color, slint::Color::from_rgb_u8(2, 132, 199));
    let btn_text = parse_hex_color(&cfg.btn_text_color, slint::Color::from_rgb_u8(245, 245, 245));

    floating_btn.set_theme_card_bg(card_bg);
    floating_btn.set_theme_card_border(card_border);
    floating_btn.set_theme_text_primary(text_primary);
    floating_btn.set_theme_text_secondary(text_secondary);
    floating_btn.set_theme_accent(accent);
    floating_btn.set_theme_btn_bg(btn_bg);
    floating_btn.set_theme_btn_text(btn_text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_clp_currency() {
        assert_eq!(format_clp_currency(""), "");
        assert_eq!(format_clp_currency("   "), "");
        assert_eq!(format_clp_currency("123123"), "$ 123.123");
        assert_eq!(format_clp_currency("1299990"), "$ 1.299.990");
        assert_eq!(format_clp_currency("$1.299.990"), "$ 1.299.990");
        assert_eq!(format_clp_currency("$ 599.990"), "$ 599.990");
        assert_eq!(format_clp_currency("84990"), "$ 84.990");
        assert_eq!(format_clp_currency("Consultar"), "Consultar");
    }

    #[test]
    fn test_format_price_input() {
        assert_eq!(format_price_input(""), "");
        assert_eq!(format_price_input("abc"), "");
        assert_eq!(format_price_input("$"), "");
        assert_eq!(format_price_input("699990"), "$ 699.990");
        assert_eq!(format_price_input("$699.990"), "$ 699.990");
        assert_eq!(format_price_input("699.990"), "$ 699.990");
        assert_eq!(format_price_input("$ 719.990"), "$ 719.990");
        assert_eq!(format_price_input("719.990"), "$ 719.990");
        assert_eq!(format_price_input("$ 990.968"), "$ 990.968");
        assert_eq!(format_price_input("990968"), "$ 990.968");
    }

    #[test]
    fn test_parse_hex_color() {
        let fallback = slint::Color::from_rgb_u8(0, 0, 0);
        let white = parse_hex_color("#ffffff", fallback);
        assert_eq!(white.red(), 255);
        assert_eq!(white.green(), 255);
        assert_eq!(white.blue(), 255);

        let blue = parse_hex_color("0284c7", fallback);
        assert_eq!(blue.red(), 2);
        assert_eq!(blue.green(), 132);
        assert_eq!(blue.blue(), 199);

        let invalid = parse_hex_color("not_a_color", fallback);
        assert_eq!(invalid.red(), 0);
        assert_eq!(invalid.green(), 0);
        assert_eq!(invalid.blue(), 0);
    }

    #[test]
    fn test_generate_analogous_gradient() {
        let (c1, c2, c3) = generate_analogous_gradient("#38bdf8");
        assert!(c1.starts_with('#'));
        assert!(c2.starts_with('#'));
        assert!(c3.starts_with('#'));
        assert_ne!(c1, c3, "Analogous colors should be distinct");

        // Verify ceiling constraint: no component greater than 245 (0xF5)
        for c in [&c1, &c2, &c3] {
            let rgb = hex_to_rgb(c).expect("Must be valid hex");
            assert!(rgb.0 <= 245, "Red must not exceed 245 (0xF5)");
            assert!(rgb.1 <= 245, "Green must not exceed 245 (0xF5)");
            assert!(rgb.2 <= 245, "Blue must not exceed 245 (0xF5)");
        }

        // Test with monochromatic color
        let (m1, m2, m3) = generate_analogous_gradient("#e0e0e0");
        assert!(m1.starts_with('#'));
        assert!(m2.starts_with('#'));
        assert!(m3.starts_with('#'));

        // Test with highly saturated light color (s > 0.70) to prevent clamp panic
        let (h1, h2, h3) = generate_analogous_gradient("#00e5ff");
        assert!(h1.starts_with('#'));
        assert!(h2.starts_with('#'));
        assert!(h3.starts_with('#'));
    }
}
