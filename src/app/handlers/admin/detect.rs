use crate::domains::hardware::detect_hardware;
use crate::MainWindow;
use slint::ComponentHandle;

pub fn setup_detect_handlers(main_window: &MainWindow) {
    let detect_handle = main_window.as_weak();
    main_window.on_auto_detect_clicked(move || {
        if let Some(ui) = detect_handle.upgrade() {
            let hw = detect_hardware();
            ui.set_setup_brand(hw.brand.clone().into());
            ui.set_setup_model(hw.model.clone().into());

            // Fill Admin form fields
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

            if !hw.others.is_empty() {
                ui.set_cfg_others(hw.others.into());
            }
        }
    });
}
