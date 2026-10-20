use crate::domains::hardware::detect_hardware;
use crate::domains::showcase::{build_ui_state, load_config};
use slint::{ModelRc, VecModel};
use crate::{MainWindow, SpecItem, VaultVideoItem};

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
    let left_items: Vec<SpecItem> = state
        .left_specs
        .iter()
        .map(|s| {
            let icon_img = slint::Image::load_from_path(std::path::Path::new(&s.icon)).unwrap_or_default();
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
            let icon_img = slint::Image::load_from_path(std::path::Path::new(&s.icon)).unwrap_or_default();
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
    let hw = detect_hardware();
    window.set_setup_brand(cfg.brand_override.clone().unwrap_or_else(|| {
        if !hw.brand.is_empty() && !hw.brand.to_lowercase().contains("to be filled") && !hw.brand.to_lowercase().contains("o.e.m.") {
            hw.brand.clone()
        } else {
            "ASUS".to_string()
        }
    }).into());

    window.set_setup_model(cfg.model_override.clone().unwrap_or_else(|| {
        if !hw.model.is_empty() && !hw.model.to_lowercase().contains("to be filled") && !hw.model.to_lowercase().contains("o.e.m.") {
            hw.model.clone()
        } else {
            "ROG Strix PC".to_string()
        }
    }).into());

    window.set_setup_retail(if !cfg.store_theme.is_empty() && !cfg.store_theme.eq_ignore_ascii_case("default") {
        cfg.store_theme.clone().into()
    } else {
        "Ripley".into()
    });
    window.set_setup_card_price(card_trim.into());
    window.set_setup_normal_price(normal_trim.into());
    window.set_setup_sku(sku_trim.into());

    // Pre-populate Admin Tienda
    window.set_cfg_brand(cfg.brand_override.clone().unwrap_or_else(|| {
        if !hw.brand.is_empty() && !hw.brand.to_lowercase().contains("to be filled") && !hw.brand.to_lowercase().contains("o.e.m.") {
            hw.brand.clone()
        } else {
            "ASUS".to_string()
        }
    }).into());

    window.set_cfg_model(cfg.model_override.clone().unwrap_or_else(|| {
        if !hw.model.is_empty() && !hw.model.to_lowercase().contains("to be filled") && !hw.model.to_lowercase().contains("o.e.m.") {
            hw.model.clone()
        } else {
            "ROG Strix PC".to_string()
        }
    }).into());

    window.set_cfg_retail(if !cfg.store_theme.is_empty() && !cfg.store_theme.eq_ignore_ascii_case("default") {
        cfg.store_theme.clone().into()
    } else {
        "Ripley".into()
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

    populate_vault_videos(window);
}
