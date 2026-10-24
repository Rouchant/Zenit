use super::types::DisplaySpecCard;
use crate::domains::hardware::SystemSpecs;
use crate::domains::showcase::config::AppConfig;

pub fn build_left_specs(specs: &SystemSpecs, cfg: &AppConfig) -> Vec<DisplaySpecCard> {
    let cpu_tag = match &cfg.badge_cpu {
        Some(custom) => {
            let trimmed = custom.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => {
            if specs.generation.is_empty() {
                None
            } else {
                Some(specs.generation.clone())
            }
        }
    };

    let ram_tag = match &cfg.badge_ram {
        Some(custom) => {
            let trimmed = custom.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => {
            if specs.ram_type.is_empty() {
                None
            } else {
                Some(specs.ram_type.clone())
            }
        }
    };

    let storage_tag = match &cfg.badge_storage {
        Some(custom) => {
            let trimmed = custom.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => {
            let low = specs.storage.to_lowercase();
            if low.contains("nvme") {
                Some("NVMe".to_string())
            } else {
                Some("SSD".to_string())
            }
        }
    };

    let os_tag = match &cfg.badge_os {
        Some(custom) => {
            let trimmed = custom.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => Some("OFICIAL".to_string()),
    };

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
    let battery_tag = match &cfg.badge_battery {
        Some(custom) => {
            let trimmed = custom.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => {
            if specs.battery.to_lowercase().contains("wh")
                || specs.battery.to_lowercase().contains("hora")
            {
                Some("Autonomía".to_string())
            } else if specs.is_charging {
                Some("Cargando".to_string())
            } else {
                Some("Autonomía".to_string())
            }
        }
    };

    let display_tag = match &cfg.badge_display {
        Some(custom) => {
            let trimmed = custom.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => {
            if specs.refresh_rate > 0 {
                Some(format!("{}Hz", specs.refresh_rate))
            } else {
                Some("165Hz".to_string())
            }
        }
    };

    let gpu_tag = match &cfg.badge_gpu {
        Some(custom) => {
            let trimmed = custom.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => {
            let low = specs.gpu.to_lowercase();
            if low.contains("rtx") {
                Some("RTX".to_string())
            } else if low.contains("gtx") {
                Some("GTX".to_string())
            } else if low.contains("radeon") || low.contains("rx") {
                Some("Radeon".to_string())
            } else if low.contains("arc") {
                Some("Arc".to_string())
            } else {
                Some("GPU".to_string())
            }
        }
    };

    let others_tag = match &cfg.badge_others {
        Some(custom) => {
            let trimmed = custom.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => None,
    };

    let others_text = if !cfg.other_specs.trim().is_empty()
        && cfg.other_specs != "Wi-Fi 6 / Bluetooth 5.3 / Teclado en Español"
    {
        cfg.other_specs.clone()
    } else if !specs.others.trim().is_empty() {
        specs.others.clone()
    } else if !cfg.other_specs.trim().is_empty() {
        cfg.other_specs.clone()
    } else {
        "Wi-Fi 6 / Bluetooth / Teclado".to_string()
    };

    vec![
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
        DisplaySpecCard {
            id: "card-others".to_string(),
            label: "Otros y Conectividad".to_string(),
            value: others_text,
            sub_value: None,
            tag: others_tag,
            icon: "assets/ui/features.svg".to_string(),
        },
    ]
}
