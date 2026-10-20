use serde::{Deserialize, Serialize};
use super::config::load_config;
use crate::domains::hardware::detect_hardware;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySpecCard {
    pub id: String,
    pub label: String,
    pub value: String,
    pub sub_value: Option<String>,
    pub tag: Option<String>,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub bg_gradient: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenitUiState {
    pub brand_name: String,
    pub brand_logo: Option<String>,
    pub store_logo: Option<String>,
    pub left_specs: Vec<DisplaySpecCard>,
    pub right_specs: Vec<DisplaySpecCard>,
    pub center_video: String,
    pub bg_video: String,
    pub screensaver_video: String,
    pub price_card: Option<String>,
    pub price_all_methods: Option<String>,
    pub sku: Option<String>,
    pub has_pricing: bool,
    pub colors: ThemeColors,
    pub inactivity_timeout_sec: u64,
    pub is_first_run: bool,
}

pub fn build_ui_state() -> ZenitUiState {
    let mut specs = detect_hardware();
    let cfg = load_config();

    // 1. Apply overrides
    if let Some(ref brand) = cfg.brand_override {
        if !brand.trim().is_empty() { specs.brand = brand.clone(); }
    }
    if let Some(ref model) = cfg.model_override {
        if !model.trim().is_empty() { specs.model = model.clone(); }
    }
    if let Some(ref proc) = cfg.processor_override {
        if !proc.trim().is_empty() { specs.processor = proc.clone(); }
    }
    if let Some(ref ram) = cfg.ram_override {
        if !ram.trim().is_empty() { specs.ram = ram.clone(); }
    }
    if let Some(ref storage) = cfg.storage_override {
        if !storage.trim().is_empty() { specs.storage = storage.clone(); }
    }
    if let Some(ref display) = cfg.display_override {
        if !display.trim().is_empty() { specs.display = display.clone(); }
    }
    if let Some(ref gpu) = cfg.gpu_override {
        if !gpu.trim().is_empty() { specs.gpu = gpu.clone(); }
    }
    if let Some(ref os) = cfg.os_override {
        if !os.trim().is_empty() { specs.os = os.clone(); }
    }
    if let Some(ref battery) = cfg.battery_override {
        let b = battery.trim();
        if !b.is_empty() {
            if let Ok(num) = b.parse::<f32>() {
                if num >= 20.0 {
                    specs.battery = format!("Batería {} Wh", num.round() as u64);
                } else {
                    specs.battery = format!("Hasta {} horas", num);
                }
            } else if b.to_lowercase().contains("wh") {
                if b.to_lowercase().starts_with("bater") {
                    specs.battery = b.to_string();
                } else {
                    specs.battery = format!("Batería {}", b);
                }
            } else if b.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
                if let Ok(num) = b.replace(',', ".").parse::<f32>() {
                    if num >= 20.0 {
                        specs.battery = format!("Batería {} Wh", num.round() as u64);
                    } else {
                        specs.battery = format!("Hasta {} horas", num);
                    }
                } else {
                    specs.battery = format!("Hasta {} horas", b);
                }
            } else if !b.to_lowercase().contains("hora")
                && !b.to_lowercase().contains("hrs")
                && !b.to_lowercase().contains("h")
                && !b.to_lowercase().contains("bater")
            {
                specs.battery = format!("Hasta {} horas", b);
            } else {
                specs.battery = b.to_string();
            }
        }
    } else {
        if specs.battery.contains("Conectado AC") || specs.battery.contains("Cargando") || specs.battery.trim().is_empty() {
            specs.battery = "Hasta 8 horas".to_string();
        }
    }

    // 2. Compute processor icon
    let proc_icon = "assets/ui/cpu.svg".to_string();

    // 3. Compute brand logo
    let brand_lower = specs.brand.to_lowercase();
    let brand_logo = if brand_lower.contains("asus") {
        Some("assets/logos/asus.svg".to_string())
    } else if brand_lower.contains("hp") {
        Some("assets/logos/hp.svg".to_string())
    } else if brand_lower.contains("lenovo") {
        Some("assets/logos/lenovo.svg".to_string())
    } else if brand_lower.contains("acer") {
        Some("assets/logos/acer.svg".to_string())
    } else if brand_lower.contains("samsung") {
        Some("assets/logos/samsung.svg".to_string())
    } else {
        None
    };

    // 4. Compute store logo
    let store_logo = match cfg.store_theme.to_lowercase().as_str() {
        "falabella" => Some("assets/logos/falabella.svg".to_string()),
        "paris" => Some("assets/logos/paris.png".to_string()),
        "ripley" => Some("assets/logos/ripley.svg".to_string()),
        _ => None,
    };

    // 5. Build Left Column Cards (Core Hardware)
    let left_specs = vec![
        DisplaySpecCard {
            id: "card-processor".to_string(),
            label: "Procesador".to_string(),
            value: specs.processor.clone(),
            sub_value: Some(format!("{} Núcleos / {} Hilos", specs.cores, specs.threads)),
            tag: if specs.generation.is_empty() { None } else { Some(specs.generation.clone()) },
            icon: proc_icon,
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
    ];

    // 6. Build Right Column Cards (Experience, Display, GPU, Battery, Model, Others)
    let full_model = if !specs.model.is_empty() && specs.model != "PC Desktop" && specs.model != "Portable PC" {
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
    let hz_tag = if specs.refresh_rate > 0 { Some(format!("{}Hz", specs.refresh_rate)) } else { None };

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
            label: "Batería".to_string(),
            value: specs.battery.clone(),
            sub_value: Some("Batería de larga duración".to_string()),
            tag: battery_tag,
            icon: "assets/ui/battery.svg".to_string(),
        },
    ];

    // Campo "Otros"
    if !cfg.other_specs.trim().is_empty() {
        right_specs.push(DisplaySpecCard {
            id: "card-others".to_string(),
            label: "Otros".to_string(),
            value: cfg.other_specs.clone(),
            sub_value: None,
            tag: None,
            icon: "assets/ui/features.svg".to_string(),
        });
    }

    // 7. Pricing & SKU Pre-computation
    let price_card = if cfg.price_card.trim().is_empty() { None } else { Some(cfg.price_card.trim().to_string()) };
    let price_all = if cfg.price_all_methods.trim().is_empty() { None } else { Some(cfg.price_all_methods.trim().to_string()) };
    let sku = if cfg.sku.trim().is_empty() { None } else { Some(cfg.sku.trim().to_string()) };
    let has_pricing = price_card.is_some() || price_all.is_some() || sku.is_some();

    ZenitUiState {
        brand_name: specs.brand,
        brand_logo,
        store_logo,
        left_specs,
        right_specs,
        center_video: cfg.central_video_path,
        bg_video: cfg.bg_video_path,
        screensaver_video: cfg.inactivity_video_path,
        price_card,
        price_all_methods: price_all,
        sku,
        has_pricing,
        colors: ThemeColors {
            primary: cfg.primary_color,
            secondary: cfg.secondary_color,
            accent: cfg.accent_color,
            bg_gradient: cfg.bg_gradient,
        },
        inactivity_timeout_sec: cfg.inactivity_timeout_sec,
        is_first_run: !cfg.setup_completed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_structure() {
        let state = build_ui_state();
        assert_eq!(state.left_specs.len(), 4, "Left column must have 4 core specs: CPU, RAM, Storage, OS");
        assert!(state.right_specs.len() >= 4, "Right column must have at least Display, GPU, Battery, Model");
        
        // Check IDs
        assert_eq!(state.left_specs[0].id, "card-processor");
        assert_eq!(state.left_specs[1].id, "card-ram");
        assert_eq!(state.left_specs[2].id, "card-storage");
        assert_eq!(state.left_specs[3].id, "card-os");

        // Verify Marca y Modelo card exists in right specs
        assert_eq!(state.right_specs[0].id, "card-model");
        assert_eq!(state.right_specs[0].label, "Marca y Modelo");
    }

    #[test]
    fn test_colors_initialization() {
        let state = build_ui_state();
        assert!(!state.colors.primary.is_empty());
        assert!(!state.colors.secondary.is_empty());
        assert!(!state.colors.accent.is_empty());
    }

    #[test]
    fn test_first_run_matches_config() {
        let cfg = load_config();
        let state = build_ui_state();
        assert_eq!(state.is_first_run, !cfg.setup_completed);
    }

    #[test]
    fn test_icons_cpu_and_software_os() {
        let state = build_ui_state();
        let cpu_card = state.left_specs.iter().find(|s| s.id == "card-processor").expect("CPU card must exist");
        let os_card = state.left_specs.iter().find(|s| s.id == "card-os").expect("OS card must exist");

        assert_eq!(cpu_card.icon, "assets/ui/cpu.svg", "CPU must use cpu.svg");
        assert_eq!(os_card.icon, "assets/ui/software.svg", "OS must use software.svg");
    }

    #[test]
    fn test_brand_model_and_battery() {
        let state = build_ui_state();
        let model_card = state.right_specs.iter().find(|s| s.id == "card-model").expect("Model card must exist");
        let battery_card = state.right_specs.iter().find(|s| s.id == "card-battery").expect("Battery card must exist");

        assert_eq!(model_card.label, "Marca y Modelo");
        assert!(!model_card.value.is_empty());
        assert!(!battery_card.value.is_empty());
        assert!(battery_card.value.to_lowercase().contains("hora") || battery_card.value.to_lowercase().contains("wh"));
    }
}
