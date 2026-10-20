use serde::{Deserialize, Serialize};
use crate::config::load_config;
use crate::hardware::detect_hardware;

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
        if !battery.trim().is_empty() { specs.battery = battery.clone(); }
    }

    // 2. Compute processor icon
    let vendor_lower = specs.vendor.to_lowercase();
    let proc_icon = if vendor_lower.contains("intel") {
        "assets/logos/intel.svg".to_string()
    } else if vendor_lower.contains("amd") {
        "assets/logos/amd.svg".to_string()
    } else {
        "assets/ui/cpu.svg".to_string()
    };

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
            icon: "assets/ui/windows-11.svg".to_string(),
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

    let battery_tag = if specs.is_charging { Some("Cargando".to_string()) } else { None };
    let hz_tag = if specs.refresh_rate > 0 { Some(format!("{}Hz", specs.refresh_rate)) } else { None };

    let mut right_specs = vec![
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
            sub_value: None,
            tag: battery_tag,
            icon: "assets/ui/battery.svg".to_string(),
        },
        DisplaySpecCard {
            id: "card-model".to_string(),
            label: "Modelo".to_string(),
            value: full_model,
            sub_value: None,
            tag: None,
            icon: "assets/ui/laptop.svg".to_string(),
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
    }
}
