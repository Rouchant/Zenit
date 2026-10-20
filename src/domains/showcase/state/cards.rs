use super::types::DisplaySpecCard;
use crate::domains::hardware::SystemSpecs;
use crate::domains::showcase::config::AppConfig;

pub fn build_left_specs(specs: &SystemSpecs) -> Vec<DisplaySpecCard> {
    vec![
        DisplaySpecCard {
            id: "card-processor".to_string(),
            label: "Procesador".to_string(),
            value: specs.processor.clone(),
            sub_value: Some(format!("{} Núcleos / {} Hilos", specs.cores, specs.threads)),
            tag: if specs.generation.is_empty() {
                None
            } else {
                Some(specs.generation.clone())
            },
            icon: "assets/ui/cpu.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-ram".to_string(),
            label: "Memoria RAM".to_string(),
            value: specs.ram.clone(),
            sub_value: None,
            tag: Some(specs.ram_type.clone()),
            icon: "assets/ui/ram.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-storage".to_string(),
            label: "Almacenamiento".to_string(),
            value: specs.storage.clone(),
            sub_value: None,
            tag: None,
            icon: "assets/ui/storage.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-os".to_string(),
            label: "Sistema Operativo".to_string(),
            value: specs.os.clone(),
            sub_value: None,
            tag: None,
            icon: "assets/ui/software.svg".to_string(),
        },
    ]
}

pub fn build_right_specs(specs: &SystemSpecs, cfg: &AppConfig) -> Vec<DisplaySpecCard> {
    let full_model = if !specs.model.is_empty()
        && specs.model != "PC Desktop"
        && specs.model != "Portable PC"
    {
        if specs.model.to_lowercase().contains(&specs.brand.to_lowercase()) {
            specs.model.clone()
        } else {
            format!("{} {}", specs.brand, specs.model)
        }
    } else {
        specs.brand.clone()
    };

    let battery_tag = if specs.battery.to_lowercase().contains("wh") {
        Some("Autonomía".to_string())
    } else if specs.battery.to_lowercase().contains("hora") {
        Some("Autonomía".to_string())
    } else if specs.is_charging {
        Some("Cargando".to_string())
    } else {
        None
    };

    let hz_tag = if specs.refresh_rate > 0 {
        Some(format!("{}Hz", specs.refresh_rate))
    } else {
        None
    };

    let model_sub_value = if !specs.brand.is_empty() && !specs.model.is_empty() {
        Some(format!("Marca: {} • Modelo: {}", specs.brand, specs.model))
    } else if !specs.brand.is_empty() {
        Some(format!("Marca: {}", specs.brand))
    } else {
        None
    };

    let brand_tag = if !specs.brand.is_empty() {
        Some(specs.brand.clone())
    } else {
        None
    };

    let others_text = if !cfg.other_specs.trim().is_empty()
        && cfg.other_specs != "Wi-Fi 6 / Bluetooth 5.3 / Teclado en Español"
    {
        cfg.other_specs.clone()
    } else if !specs.others.trim().is_empty() {
        specs.others.clone()
    } else {
        cfg.other_specs.clone()
    };

    let mut right_specs = vec![
        DisplaySpecCard {
            id: "card-model".to_string(),
            label: "Marca y Modelo".to_string(),
            value: full_model,
            sub_value: model_sub_value,
            tag: brand_tag,
            icon: "assets/ui/laptop.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-display".to_string(),
            label: "Pantalla".to_string(),
            value: specs.display.clone(),
            sub_value: None,
            tag: hz_tag,
            icon: "assets/ui/screen.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-gpu".to_string(),
            label: "Gráficos".to_string(),
            value: specs.gpu.clone(),
            sub_value: None,
            tag: None,
            icon: "assets/ui/gpu.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-battery".to_string(),
            label: "Batería y Conectividad".to_string(),
            value: specs.battery.clone(),
            sub_value: Some(others_text.clone()),
            tag: battery_tag,
            icon: "assets/ui/battery.svg".to_string(),
        },
    ];

    if !others_text.trim().is_empty() {
        right_specs.push(DisplaySpecCard {
            id: "card-others".to_string(),
            label: "Otros".to_string(),
            value: others_text,
            sub_value: None,
            tag: None,
            icon: "assets/ui/features.svg".to_string(),
        });
    }

    right_specs
}
