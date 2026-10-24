use crate::app::sync::populate_ui_state;
use crate::domains::hardware::get_hardware_specs;
use crate::MainWindow;
use slint::ComponentHandle;

pub fn setup_defaults_handlers(main_window: &MainWindow) {
    // 1. Reset Store Defaults
    let reset_store_handle = main_window.as_weak();
    main_window.on_reset_store_defaults(move || {
        if let Some(ui) = reset_store_handle.upgrade() {
            let hw = get_hardware_specs();
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
            ui.set_cfg_brand_logo("".into());
            ui.set_cfg_model(model.into());
            ui.set_cfg_retail("".into());
            ui.set_cfg_store_logo("".into());
            ui.set_cfg_card_price("".into());
            ui.set_cfg_normal_price("".into());
            ui.set_cfg_sku("".into());
        }
    });

    // 2. Reset Specs Defaults
    let reset_specs_handle = main_window.as_weak();
    main_window.on_reset_specs_defaults(move || {
        if let Some(ui) = reset_specs_handle.upgrade() {
            let hw = get_hardware_specs();
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
            ui.set_cfg_cpu(hw.processor.clone().into());
            ui.set_cfg_ram(hw.ram.clone().into());
            ui.set_cfg_storage(hw.storage.clone().into());
            ui.set_cfg_gpu(hw.gpu.clone().into());
            ui.set_cfg_display(hw.display.clone().into());
            ui.set_cfg_os(hw.os.clone().into());
            ui.set_cfg_battery(if !hw.battery.is_empty() && !hw.battery.contains("Conectado AC") {
                hw.battery.into()
            } else {
                "Hasta 8 horas".into()
            });
            let others_val = if !hw.others.is_empty() {
                hw.others.clone()
            } else {
                "Wi-Fi 6 / Bluetooth 5.3 / Teclado en Español".to_string()
            };
            ui.set_cfg_others(others_val.into());

            // Badges por defecto detectadas insitu
            let def_badge_cpu = if !hw.generation.is_empty() {
                hw.generation.clone()
            } else {
                "GEN".to_string()
            };
            let def_badge_ram = if !hw.ram_type.is_empty() {
                hw.ram_type.clone()
            } else {
                "RAM".to_string()
            };
            let def_badge_storage = if hw.storage.to_lowercase().contains("nvme") {
                "NVMe".to_string()
            } else {
                "SSD".to_string()
            };
            let def_badge_gpu = {
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
            };
            let def_badge_display = if hw.refresh_rate > 0 {
                format!("{}Hz", hw.refresh_rate)
            } else {
                "165Hz".to_string()
            };

            ui.set_cfg_badge_cpu(def_badge_cpu.into());
            ui.set_cfg_badge_ram(def_badge_ram.into());
            ui.set_cfg_badge_storage(def_badge_storage.into());
            ui.set_cfg_badge_gpu(def_badge_gpu.into());
            ui.set_cfg_badge_display(def_badge_display.into());
            ui.set_cfg_badge_os("OFICIAL".into());
            ui.set_cfg_badge_battery("Autonomía".into());
            ui.set_cfg_badge_others("".into());
        }
    });

    // 3. Reset Videos Defaults
    let reset_videos_handle = main_window.as_weak();
    main_window.on_reset_videos_defaults(move || {
        if let Some(ui) = reset_videos_handle.upgrade() {
            let hw = get_hardware_specs();
            let brand_str = ui.get_cfg_brand().to_string();
            let gpu_str = ui.get_cfg_gpu().to_string();
            let effective_brand = if !brand_str.trim().is_empty() { brand_str } else { hw.brand.clone() };
            let effective_gpu = if !gpu_str.trim().is_empty() { gpu_str } else { hw.gpu.clone() };

            let (def_central, def_inact_1, def_inact_2) =
                crate::domains::showcase::config::compute_default_video_paths(&effective_brand, &effective_gpu);

            ui.set_cfg_central_video(def_central.clone().into());
            ui.set_cfg_bg_video(def_central.into());
            ui.set_cfg_inactivity_video_1(def_inact_1.into());
            ui.set_cfg_inactivity_video_2(def_inact_2.into());
            ui.set_cfg_inactivity_video_3("".into());
            ui.set_cfg_inactivity_timeout("120".into());
        }
    });

    // 4. Reset System Defaults
    let reset_sys_handle = main_window.as_weak();
    main_window.on_reset_system_defaults(move || {
        if let Some(ui) = reset_sys_handle.upgrade() {
            let default_cfg = crate::domains::showcase::config::AppConfig::default();
            let _ = crate::domains::showcase::config::save_config(&default_cfg);
            populate_ui_state(&ui);
        }
    });

    // 5. Reset UI Theme Defaults (Pastel Zenit)
    let reset_ui_handle = main_window.as_weak();
    main_window.on_reset_ui_defaults(move || {
        if let Some(ui) = reset_ui_handle.upgrade() {
            ui.set_cfg_bg_base("#cce5f8".into());
            let (c1, c2, c3) = crate::app::sync::generate_analogous_gradient("#cce5f8");
            ui.set_cfg_bg_start(c1.into());
            ui.set_cfg_bg_mid(c2.into());
            ui.set_cfg_bg_end(c3.into());
            ui.set_cfg_card_bg("#F5F5F5".into());
            ui.set_cfg_card_border("#bae6fd".into());
            ui.set_cfg_text_primary("#0f172a".into());
            ui.set_cfg_text_secondary("#475569".into());
            ui.set_cfg_accent("#0284c7".into());
            ui.set_cfg_btn_bg("#0284c7".into());
            ui.set_cfg_btn_text("#F5F5F5".into());
            ui.set_is_dark_mode(false);
        }
    });
}
