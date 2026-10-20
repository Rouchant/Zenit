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
            ui.set_cfg_model(model.into());
            ui.set_cfg_retail("ripley".into());
            ui.set_cfg_card_price("$1.299.990".into());
            ui.set_cfg_normal_price("$1.349.990".into());
            ui.set_cfg_sku("8493021".into());
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
        }
    });

    // 3. Reset Videos Defaults
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

    // 4. Reset System Defaults
    let reset_sys_handle = main_window.as_weak();
    main_window.on_reset_system_defaults(move || {
        if let Some(ui) = reset_sys_handle.upgrade() {
            let default_cfg = crate::domains::showcase::config::AppConfig::default();
            let _ = crate::domains::showcase::config::save_config(&default_cfg);
            populate_ui_state(&ui);
        }
    });
}
