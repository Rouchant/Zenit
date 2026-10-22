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
            let others_val = if !hw.others.is_empty() {
                hw.others.clone()
            } else {
                "Wi-Fi 6 / Bluetooth 5.3 / Teclado en Español".to_string()
            };
            ui.set_cfg_others(others_val.into());

            ui.set_cfg_badge_cpu("".into());
            ui.set_cfg_badge_ram("".into());
            ui.set_cfg_badge_storage("".into());
            ui.set_cfg_badge_gpu("".into());
            ui.set_cfg_badge_display("".into());
            ui.set_cfg_badge_os("".into());
            ui.set_cfg_badge_battery("".into());
            ui.set_cfg_badge_others("".into());
        }
    });

    // 3. Reset Videos Defaults
    let reset_videos_handle = main_window.as_weak();
    main_window.on_reset_videos_defaults(move || {
        if let Some(ui) = reset_videos_handle.upgrade() {
            ui.set_cfg_central_video("assets/videos/landing2.mp4".into());
            ui.set_cfg_bg_video("assets/videos/bg.mp4".into());
            ui.set_cfg_inactivity_video_1("assets/videos/promo2.mp4".into());
            ui.set_cfg_inactivity_video_2("".into());
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

    // 5. Reset UI Theme Defaults
    let reset_ui_handle = main_window.as_weak();
    main_window.on_reset_ui_defaults(move || {
        if let Some(ui) = reset_ui_handle.upgrade() {
            ui.set_cfg_bg_base("#e3e8f5".into());
            let (c1, c2, c3) = crate::app::sync::generate_analogous_gradient("#e3e8f5");
            ui.set_cfg_bg_start(c1.into());
            ui.set_cfg_bg_mid(c2.into());
            ui.set_cfg_bg_end(c3.into());
            ui.set_cfg_card_bg("#F5F5F5".into());
            ui.set_cfg_card_border("#e2e8f0".into());
            ui.set_cfg_text_primary("#0f172a".into());
            ui.set_cfg_text_secondary("#64748b".into());
            ui.set_cfg_accent("#0284c7".into());
            ui.set_cfg_btn_bg("#0284c7".into());
            ui.set_cfg_btn_text("#F5F5F5".into());
            ui.set_is_dark_mode(false);
        }
    });
}
