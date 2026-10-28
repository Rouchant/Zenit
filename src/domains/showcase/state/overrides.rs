use crate::domains::hardware::SystemSpecs;
use crate::domains::showcase::config::AppConfig;

pub fn apply_config_overrides(specs: &mut SystemSpecs, cfg: &AppConfig) {
    if let Some(ref brand) = cfg.brand_override {
        if !brand.trim().is_empty() {
            specs.brand = brand.clone();
        }
    }
    if let Some(ref model) = cfg.model_override {
        if !model.trim().is_empty() {
            specs.model = model.clone();
        }
    }
    if let Some(ref proc) = cfg.processor_override {
        if !proc.trim().is_empty() {
            specs.processor = proc.clone();
        }
    }
    if let Some(ref ram) = cfg.ram_override {
        if !ram.trim().is_empty() {
            specs.ram = ram.clone();
        }
    }
    if let Some(ref storage) = cfg.storage_override {
        if !storage.trim().is_empty() {
            specs.storage = storage.clone();
        }
    }
    if let Some(ref display) = cfg.display_override {
        if !display.trim().is_empty() {
            specs.display = display.clone();
        }
    }
    if let Some(ref gpu) = cfg.gpu_override {
        if !gpu.trim().is_empty() {
            specs.gpu = gpu.clone();
        }
    }
    if let Some(ref os) = cfg.os_override {
        if !os.trim().is_empty() {
            specs.os = os.clone();
        }
    }
    if let Some(ref battery) = cfg.battery_override {
        let b = battery.trim();
        // Si el override guardado era el antiguo valor por defecto "Hasta 8 horas",
        // priorizar la capacidad en Wh detectada en el equipo.
        if !b.is_empty() && !b.eq_ignore_ascii_case("Hasta 8 horas") {
            if let Ok(num) = b.parse::<f32>() {
                specs.battery = format!("Batería {} Wh", num.round() as u64);
            } else if b.to_lowercase().contains("wh") {
                if b.to_lowercase().starts_with("bater") {
                    specs.battery = b.to_string();
                } else {
                    specs.battery = format!("Batería {}", b);
                }
            } else if b.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
                if let Ok(num) = b.replace(',', ".").parse::<f32>() {
                    specs.battery = format!("Batería {} Wh", num.round() as u64);
                } else {
                    specs.battery = format!("Batería {} Wh", b);
                }
            } else {
                specs.battery = b.to_string();
            }
        }
    } else if specs.battery.contains("Conectado AC")
        || specs.battery.contains("Cargando")
        || specs.battery.trim().is_empty()
    {
        specs.battery = "Batería 50 Wh".to_string();
    }
}
