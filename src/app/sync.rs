use crate::domains::hardware::{get_hardware_specs, is_oem_placeholder};
use crate::domains::showcase::{build_ui_state, load_config};
use slint::{ModelRc, VecModel};
use crate::{MainWindow, SpecItem, VaultVideoItem};
use std::cell::RefCell;
use std::collections::HashMap;

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
        .map(|v| VaultVideoItem {
            name: v.name.into(),
            filename: v.filename.into(),
            path: v.path.into(),
            size_str: format!("{:.1} MB", v.size_mb).into(),
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
        if !state.right_specs.is_empty() {
            state
                .right_specs
                .iter()
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
    window.set_cfg_model(clean_model.into());

    window.set_cfg_retail(if !cfg.store_theme.is_empty() && !cfg.store_theme.eq_ignore_ascii_case("default") {
        cfg.store_theme.clone().into()
    } else {
        "".into()
    });
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
        state.right_specs.iter()
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
        state.left_specs.iter()
            .find(|s| s.id == "card-os")
            .map(|s| s.value.clone())
            .unwrap_or_else(|| hw.os.clone())
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

    // Admin Videos
    window.set_cfg_central_video(cfg.central_video_path.clone().into());
    window.set_cfg_bg_video(cfg.bg_video_path.clone().into());
    window.set_cfg_inactivity_video_1(cfg.inactivity_video_path.clone().into());
    window.set_cfg_inactivity_video_2(cfg.inactivity_video_path_2.clone().into());
    window.set_cfg_inactivity_video_3(cfg.inactivity_video_path_3.clone().into());
    window.set_cfg_inactivity_timeout(cfg.inactivity_timeout_sec.to_string().into());

    // Theme Colors Synchronization
    let bg_start = parse_hex_color(&cfg.bg_start_color, slint::Color::from_rgb_u8(224, 242, 254)); // #e0f2fe
    let bg_end = parse_hex_color(&cfg.bg_end_color, slint::Color::from_rgb_u8(243, 232, 255)); // #f3e8ff
    let card_bg = parse_hex_color(&cfg.card_bg_color, slint::Color::from_rgb_u8(255, 255, 255));
    let card_border = parse_hex_color(&cfg.card_border_color, slint::Color::from_rgb_u8(226, 232, 240));
    let text_primary = parse_hex_color(&cfg.text_primary_color, slint::Color::from_rgb_u8(15, 23, 42));
    let text_secondary = parse_hex_color(&cfg.text_secondary_color, slint::Color::from_rgb_u8(100, 116, 139));
    let accent = parse_hex_color(&cfg.accent_color, slint::Color::from_rgb_u8(2, 132, 199));
    let btn_bg = parse_hex_color(&cfg.btn_bg_color, slint::Color::from_rgb_u8(2, 132, 199));
    let btn_text = parse_hex_color(&cfg.btn_text_color, slint::Color::from_rgb_u8(255, 255, 255));

    window.set_theme_bg_start(bg_start);
    window.set_theme_bg_end(bg_end);
    window.set_theme_card_bg(card_bg);
    window.set_theme_card_border(card_border);
    window.set_theme_text_primary(text_primary);
    window.set_theme_text_secondary(text_secondary);
    window.set_theme_accent(accent);
    window.set_theme_btn_bg(btn_bg);
    window.set_theme_btn_text(btn_text);

    // Admin Interfaz Strings
    window.set_cfg_bg_start(cfg.bg_start_color.clone().into());
    window.set_cfg_bg_end(cfg.bg_end_color.clone().into());
    window.set_cfg_card_bg(cfg.card_bg_color.clone().into());
    window.set_cfg_card_border(cfg.card_border_color.clone().into());
    window.set_cfg_text_primary(cfg.text_primary_color.clone().into());
    window.set_cfg_text_secondary(cfg.text_secondary_color.clone().into());
    window.set_cfg_accent(cfg.accent_color.clone().into());
    window.set_cfg_btn_bg(cfg.btn_bg_color.clone().into());
    window.set_cfg_btn_text(cfg.btn_text_color.clone().into());

    populate_vault_videos(window);
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
}
