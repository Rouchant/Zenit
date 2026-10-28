use crate::domains::hardware::{is_oem_placeholder, refresh_hardware_specs};
use crate::MainWindow;
use slint::ComponentHandle;

pub fn setup_detect_handlers(main_window: &MainWindow) {
    let detect_handle = main_window.as_weak();
    main_window.on_auto_detect_clicked(move || {
        let dh = detect_handle.clone();
        std::thread::spawn(move || {
            let hw = refresh_hardware_specs();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = dh.upgrade() {
                    let brand = if !is_oem_placeholder(&hw.brand) {
                        hw.brand.clone()
                    } else {
                        "ASUS".to_string()
                    };
                    let model = if !is_oem_placeholder(&hw.model) {
                        hw.model.clone()
                    } else {
                        "ROG Strix PC".to_string()
                    };

                    ui.set_setup_brand(brand.clone().into());
                    ui.set_setup_model(model.clone().into());

                    // Fill Admin form fields
                    ui.set_cfg_brand(brand.into());
                    ui.set_cfg_model(model.into());
                    ui.set_cfg_cpu(hw.processor.into());
                    ui.set_cfg_ram(hw.ram.into());
                    ui.set_cfg_storage(hw.storage.into());
                    ui.set_cfg_gpu(hw.gpu.into());
                    ui.set_cfg_display(hw.display.into());
                    ui.set_cfg_os(hw.os.into());

                    if hw.battery.contains("Conectado AC") || hw.battery.contains("Cargando") {
                        if ui.get_cfg_battery().trim().is_empty() {
                            ui.set_cfg_battery("Batería 50 Wh".into());
                        }
                    } else {
                        ui.set_cfg_battery(hw.battery.into());
                    }

                    if !hw.others.is_empty() {
                        ui.set_cfg_others(hw.others.into());
                    }
                }
            });
        });
    });
}
