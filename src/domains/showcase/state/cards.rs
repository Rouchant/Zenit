use super::types::DisplaySpecCard;
use crate::domains::hardware::SystemSpecs;
use crate::domains::showcase::config::AppConfig;

pub fn build_left_specs(specs: &SystemSpecs, cfg: &AppConfig) -> Vec<DisplaySpecCard> {
    let cpu_tag = cfg
        .badge_cpu
        .as_ref()
        .filter(|b| !b.trim().is_empty())
        .cloned()
        .or_else(|| {
            if specs.generation.is_empty() {
                None
            } else {
                Some(specs.generation.clone())
            }
        });

    let ram_tag = cfg
        .badge_ram
        .as_ref()
        .filter(|b| !b.trim().is_empty())
        .cloned()
        .or_else(|| {
            if specs.ram_type.is_empty() {
                None
            } else {
                Some(specs.ram_type.clone())
            }
        });

    let storage_tag = cfg
        .badge_storage
        .as_ref()
        .filter(|b| !b.trim().is_empty())
        .cloned();

    let os_tag = cfg
        .badge_os
        .as_ref()
        .filter(|b| !b.trim().is_empty())
        .cloned();

    vec![
        DisplaySpecCard {
            id: "card-processor".to_string(),
            label: "Procesador".to_string(),
            value: specs.processor.clone(),
            sub_value: Some(format!("{} Núcleos / {} Hilos", specs.cores, specs.threads)),
            tag: cpu_tag,
            icon: "assets/ui/cpu.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-ram".to_string(),
            label: "Memoria RAM".to_string(),
            value: specs.ram.clone(),
            sub_value: None,
            tag: ram_tag,
            icon: "assets/ui/ram.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-storage".to_string(),
            label: "Almacenamiento".to_string(),
            value: specs.storage.clone(),
            sub_value: None,
            tag: storage_tag,
            icon: "assets/ui/storage.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-os".to_string(),
            label: "Sistema Operativo".to_string(),
            value: specs.os.clone(),
            sub_value: None,
            tag: os_tag,
            icon: "assets/ui/software.svg".to_string(),
        },
    ]
}

pub fn build_right_specs(specs: &SystemSpecs, cfg: &AppConfig) -> Vec<DisplaySpecCard> {
    let model_text = if !specs.model.is_empty() {
        specs.model.clone()
    } else {
        "PC Showroom".to_string()
    };

    let brand_tag = if !specs.brand.is_empty() {
        Some(specs.brand.clone())
    } else {
        None
    };

    let battery_tag = cfg
        .badge_battery
        .as_ref()
        .filter(|b| !b.trim().is_empty())
        .cloned()
        .or_else(|| {
            if specs.battery.to_lowercase().contains("wh") {
                Some("Autonomía".to_string())
            } else if specs.battery.to_lowercase().contains("hora") {
                Some("Autonomía".to_string())
            } else if specs.is_charging {
                Some("Cargando".to_string())
            } else {
                None
            }
        });

    let display_tag = cfg
        .badge_display
        .as_ref()
        .filter(|b| !b.trim().is_empty())
        .cloned()
        .or_else(|| {
            if specs.refresh_rate > 0 {
                Some(format!("{}Hz", specs.refresh_rate))
            } else {
                None
            }
        });

    let gpu_tag = cfg
        .badge_gpu
        .as_ref()
        .filter(|b| !b.trim().is_empty())
        .cloned();

    let others_tag = cfg
        .badge_others
        .as_ref()
        .filter(|b| !b.trim().is_empty())
        .cloned();

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
            value: model_text,
            sub_value: None,
            tag: brand_tag,
            icon: "assets/ui/laptop.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-display".to_string(),
            label: "Pantalla".to_string(),
            value: specs.display.clone(),
            sub_value: None,
            tag: display_tag,
            icon: "assets/ui/screen.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-gpu".to_string(),
            label: "Gráficos".to_string(),
            value: specs.gpu.clone(),
            sub_value: None,
            tag: gpu_tag,
            icon: "assets/ui/gpu.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-battery".to_string(),
            label: "Batería".to_string(),
            value: specs.battery.clone(),
            sub_value: None,
            tag: battery_tag,
            icon: "assets/ui/battery.svg".to_string(),
        },
    ];

    if !others_text.trim().is_empty() {
        right_specs.push(DisplaySpecCard {
            id: "card-others".to_string(),
            label: "Otros y Conectividad".to_string(),
            value: others_text,
            sub_value: None,
            tag: others_tag,
            icon: "assets/ui/features.svg".to_string(),
        });
    }

    right_specs
}
