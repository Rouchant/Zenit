use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

fn default_bg_base() -> String {
    "#cce5f8".to_string()
}
fn default_bg_start() -> String {
    "#c4e9f8".to_string()
}
fn default_bg_mid() -> String {
    "#cce5f8".to_string()
}
fn default_bg_end() -> String {
    "#dce2f8".to_string()
}
fn default_card_bg() -> String {
    "#F5F5F5".to_string()
}
fn default_card_border() -> String {
    "#bae6fd".to_string()
}
fn default_text_primary() -> String {
    "#0f172a".to_string()
}
fn default_text_secondary() -> String {
    "#475569".to_string()
}
fn default_accent() -> String {
    "#0284c7".to_string()
}
fn default_btn_bg() -> String {
    "#0284c7".to_string()
}
fn default_btn_text() -> String {
    "#F5F5F5".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Theme & Colors
    pub primary_color: String,
    pub secondary_color: String,
    #[serde(default = "default_accent")]
    pub accent_color: String,
    pub bg_gradient: String,
    pub store_theme: String, // "default", "falabella", "paris", "ripley"

    #[serde(default = "default_bg_base")]
    pub bg_base_color: String,
    #[serde(default = "default_bg_start")]
    pub bg_start_color: String,
    #[serde(default = "default_bg_mid")]
    pub bg_mid_color: String,
    #[serde(default = "default_bg_end")]
    pub bg_end_color: String,
    #[serde(default = "default_card_bg")]
    pub card_bg_color: String,
    #[serde(default = "default_card_border")]
    pub card_border_color: String,
    #[serde(default = "default_text_primary")]
    pub text_primary_color: String,
    #[serde(default = "default_text_secondary")]
    pub text_secondary_color: String,
    #[serde(default = "default_btn_bg")]
    pub btn_bg_color: String,
    #[serde(default = "default_btn_text")]
    pub btn_text_color: String,
    #[serde(default)]
    pub is_dark_mode: bool,

    // Optional Pricing & SKU
    pub price_card: String,
    pub price_all_methods: String,
    pub sku: String,

    // Custom Spec Overrides
    pub brand_override: Option<String>,
    #[serde(default)]
    pub brand_logo_override: Option<String>,
    #[serde(default)]
    pub store_logo_override: Option<String>,
    pub model_override: Option<String>,
    pub processor_override: Option<String>,
    pub ram_override: Option<String>,
    pub storage_override: Option<String>,
    pub display_override: Option<String>,
    pub gpu_override: Option<String>,
    pub os_override: Option<String>,
    pub battery_override: Option<String>,
    pub other_specs: String, // Campo "Otros" personalizable
    #[serde(default)]
    pub badge_cpu: Option<String>,
    #[serde(default)]
    pub badge_ram: Option<String>,
    #[serde(default)]
    pub badge_storage: Option<String>,
    #[serde(default)]
    pub badge_gpu: Option<String>,
    #[serde(default)]
    pub badge_display: Option<String>,
    #[serde(default)]
    pub badge_os: Option<String>,
    #[serde(default)]
    pub badge_battery: Option<String>,
    #[serde(default)]
    pub badge_others: Option<String>,

    // Videos
    pub central_video_path: String,
    pub bg_video_path: String,
    pub inactivity_video_path: String,
    #[serde(default)]
    pub inactivity_video_path_2: String,
    #[serde(default)]
    pub inactivity_video_path_3: String,
    pub inactivity_timeout_sec: u64,

    // First Run Setup
    #[serde(default)]
    pub setup_completed: bool,
}

impl AppConfig {
    pub fn get_inactivity_video_list(&self) -> Vec<String> {
        let mut list = Vec::new();
        if !self.inactivity_video_path.trim().is_empty() {
            list.push(self.inactivity_video_path.clone());
        }
        if !self.inactivity_video_path_2.trim().is_empty() {
            list.push(self.inactivity_video_path_2.clone());
        }
        if !self.inactivity_video_path_3.trim().is_empty() {
            list.push(self.inactivity_video_path_3.clone());
        }
        if list.is_empty() {
            list.push("assets/videos/generic/promo-generic.mp4".to_string());
        }
        list
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            primary_color: "#00f2ff".to_string(),
            secondary_color: "#0072ff".to_string(),
            accent_color: "#0284c7".to_string(),
            bg_gradient: "@linear-gradient(155deg, #c4e9f8 0%, #cce5f8 50%, #dce2f8 100%)".to_string(),
            store_theme: "default".to_string(),

            bg_base_color: "#cce5f8".to_string(),
            bg_start_color: "#c4e9f8".to_string(),
            bg_mid_color: "#cce5f8".to_string(),
            bg_end_color: "#dce2f8".to_string(),
            card_bg_color: "#F5F5F5".to_string(),
            card_border_color: "#bae6fd".to_string(),
            text_primary_color: "#0f172a".to_string(),
            text_secondary_color: "#475569".to_string(),
            btn_bg_color: "#0284c7".to_string(),
            btn_text_color: "#F5F5F5".to_string(),
            is_dark_mode: false,

            price_card: "".to_string(),
            price_all_methods: "".to_string(),
            sku: "".to_string(),

            brand_override: None,
            brand_logo_override: None,
            store_logo_override: None,
            model_override: None,
            processor_override: None,
            ram_override: None,
            storage_override: None,
            display_override: None,
            gpu_override: None,
            os_override: None,
            battery_override: None,
            other_specs: "Wi-Fi 6 / Bluetooth 5.3 / Teclado en Español".to_string(),
            badge_cpu: None,
            badge_ram: None,
            badge_storage: None,
            badge_gpu: None,
            badge_display: None,
            badge_os: None,
            badge_battery: None,
            badge_others: None,

            central_video_path: "assets/videos/generic/landing-generic.mp4".to_string(),
            bg_video_path: "assets/videos/generic/landing-generic.mp4".to_string(),
            inactivity_video_path: "assets/videos/generic/promo-generic.mp4".to_string(),
            inactivity_video_path_2: "".to_string(),
            inactivity_video_path_3: "".to_string(),
            inactivity_timeout_sec: 120,

            setup_completed: false,
        }
    }
}

fn get_config_file_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = Path::new(&app_data).join("Zenit");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir.join("config.json")
}

use std::sync::RwLock;

static CACHED_CONFIG: RwLock<Option<AppConfig>> = RwLock::new(None);

pub fn load_config() -> AppConfig {
    if let Ok(guard) = CACHED_CONFIG.read() {
        if let Some(ref cfg) = *guard {
            return cfg.clone();
        }
    }

    let cfg = load_config_from_disk();
    if let Ok(mut guard) = CACHED_CONFIG.write() {
        *guard = Some(cfg.clone());
    }
    cfg
}

/// Calcula los videos por defecto basados en el hardware detectado:
/// - Slot 1 Promo: assets/videos/generic/promo-generic.mp4 (en cualquier equipo)
/// - Slot 2 Promo: assets/videos/brand_a/v_a_p.mp4 (si el equipo es ASUS)
/// - Landing por defecto:
///     - Si detecta RTX: assets/videos/generic/gaming_xbox_game_pass.mp4
///     - En cualquier otro caso: assets/videos/generic/landing-generic.mp4
pub fn compute_default_video_paths(brand: &str, gpu: &str) -> (String, String, String) {
    let is_rtx = gpu.to_lowercase().contains("rtx");
    let is_asus = brand.to_lowercase().contains("asus");

    let central_video = if is_rtx {
        "assets/videos/generic/gaming_xbox_game_pass.mp4".to_string()
    } else {
        "assets/videos/generic/landing-generic.mp4".to_string()
    };

    let inactivity_1 = "assets/videos/generic/promo-generic.mp4".to_string();

    let inactivity_2 = if is_asus {
        "assets/videos/brand_a/v_a_p.mp4".to_string()
    } else {
        "".to_string()
    };

    (central_video, inactivity_1, inactivity_2)
}

fn load_config_from_disk() -> AppConfig {
    let path = get_config_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&content) {
                let mut modified = false;

                let hw = crate::domains::hardware::get_hardware_specs();
                let effective_brand = cfg.brand_override.as_deref().unwrap_or(&hw.brand);
                let effective_gpu = cfg.gpu_override.as_deref().unwrap_or(&hw.gpu);
                let (def_central, def_inact_1, def_inact_2) = compute_default_video_paths(effective_brand, effective_gpu);

                if cfg.central_video_path == "assets/videos/landing.mp4"
                    || cfg.central_video_path == "assets/videos/landing2.mp4"
                    || cfg.central_video_path.is_empty()
                {
                    cfg.central_video_path = def_central.clone();
                    modified = true;
                }
                if cfg.inactivity_video_path == "assets/videos/promo.mp4"
                    || cfg.inactivity_video_path == "assets/videos/promo2.mp4"
                    || cfg.inactivity_video_path.is_empty()
                {
                    cfg.inactivity_video_path = def_inact_1;
                    modified = true;
                }
                if effective_brand.to_lowercase().contains("asus")
                    && (cfg.inactivity_video_path_2.is_empty() || cfg.inactivity_video_path_2 == "assets/videos/promo2.mp4")
                {
                    cfg.inactivity_video_path_2 = def_inact_2;
                    modified = true;
                }
                if cfg.bg_video_path == "assets/images/fallback-bg/bg.mp4"
                    || cfg.bg_video_path == "assets/videos/bg.mp4"
                    || cfg.bg_video_path.is_empty()
                {
                    cfg.bg_video_path = def_central;
                    modified = true;
                }
                let is_oem_placeholder = |s: &str| -> bool {
                    let l = s.to_lowercase();
                    l.contains("to be filled")
                        || l.contains("system manufacturer")
                        || l.contains("system product name")
                        || l.contains("o.e.m.")
                        || l.contains("default string")
                        || l.trim().is_empty()
                };

                if let Some(ref b) = cfg.brand_override {
                    if is_oem_placeholder(b) {
                        cfg.brand_override = None;
                        modified = true;
                    }
                }
                if let Some(ref m) = cfg.model_override {
                    if is_oem_placeholder(m) {
                        cfg.model_override = None;
                        modified = true;
                    }
                }

                if cfg.price_card == "$1.299.990" || cfg.price_card == "$ 1.299.990" {
                    cfg.price_card = "".to_string();
                    modified = true;
                }
                if cfg.price_all_methods == "$1.349.990" || cfg.price_all_methods == "$ 1.349.990" {
                    cfg.price_all_methods = "".to_string();
                    modified = true;
                }
                if cfg.sku == "8493021" {
                    cfg.sku = "".to_string();
                    modified = true;
                }
                if cfg.store_theme.eq_ignore_ascii_case("ripley") { cfg.store_theme = "".to_string(); modified = true; } if cfg.battery_override.as_deref() == Some("Hasta 8 horas") { cfg.battery_override = None; modified = true; } if false {
                    cfg.store_theme = "".to_string();
                    modified = true;
                }
                if (cfg.bg_start_color == "#f8fafc" && cfg.bg_end_color == "#e2e8f0")
                    || (cfg.bg_start_color == "#e0f2fe" && cfg.bg_end_color == "#f3e8ff")
                    || (cfg.bg_start_color == "#f0f9ff" && cfg.bg_end_color == "#faf5ff")
                    || (cfg.bg_start_color == "#e0ecf5" && cfg.bg_end_color == "#eae4f5")
                {
                    cfg.bg_base_color = "#cce5f8".to_string();
                    cfg.bg_start_color = "#c4e9f8".to_string();
                    cfg.bg_mid_color = "#cce5f8".to_string();
                    cfg.bg_end_color = "#dce2f8".to_string();
                    modified = true;
                }
                if cfg.card_bg_color.eq_ignore_ascii_case("#ffffff") || cfg.card_bg_color.eq_ignore_ascii_case("#E0E0E0") {
                    cfg.card_bg_color = "#F5F5F5".to_string();
                    modified = true;
                }
                if cfg.btn_text_color.eq_ignore_ascii_case("#ffffff") || cfg.btn_text_color.eq_ignore_ascii_case("#E0E0E0") {
                    cfg.btn_text_color = "#F5F5F5".to_string();
                    modified = true;
                }
                if cfg.bg_start_color.eq_ignore_ascii_case("#ffffff") || cfg.bg_start_color.eq_ignore_ascii_case("#E0E0E0") {
                    cfg.bg_start_color = "#F5F5F5".to_string();
                    modified = true;
                }
                if cfg.bg_end_color.eq_ignore_ascii_case("#ffffff") || cfg.bg_end_color.eq_ignore_ascii_case("#E0E0E0") {
                    cfg.bg_end_color = "#F5F5F5".to_string();
                    modified = true;
                }

                if modified {
                    let _ = save_config(&cfg);
                }
                return cfg;
            }
        }
    }
    let mut default_cfg = AppConfig::default();
    let hw = crate::domains::hardware::get_hardware_specs();
    let (def_central, def_inact_1, def_inact_2) = compute_default_video_paths(&hw.brand, &hw.gpu);
    default_cfg.central_video_path = def_central.clone();
    default_cfg.bg_video_path = def_central;
    default_cfg.inactivity_video_path = def_inact_1;
    default_cfg.inactivity_video_path_2 = def_inact_2;
    let _ = save_config(&default_cfg);
    default_cfg
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_file_path();
    let data = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Error al serializar config: {}", e))?;
    fs::write(path, data)
        .map_err(|e| format!("Error al guardar config en disco: {}", e))?;

    if let Ok(mut guard) = CACHED_CONFIG.write() {
        *guard = Some(config.clone());
    }
    Ok(())
}

pub fn verify_password(input: &str) -> bool {
    let clean = input.trim().to_lowercase();
    clean == "demo" || clean == "z3n1t"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_password_demo() {
        assert!(verify_password("demo"));
        assert!(verify_password("DEMO"));
        assert!(verify_password("  Demo  "));
    }

    #[test]
    fn test_verify_password_master() {
        assert!(verify_password("z3n1t"));
        assert!(verify_password("Z3N1T"));
        assert!(verify_password("  z3N1T  "));
    }

    #[test]
    fn test_verify_password_invalid() {
        assert!(!verify_password("wrong"));
        assert!(!verify_password(""));
        assert!(!verify_password("1234"));
        assert!(!verify_password("admin"));
    }

    #[test]
    fn test_app_config_defaults() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.primary_color, "#00f2ff");
        assert_eq!(cfg.store_theme, "default");
        assert_eq!(cfg.central_video_path, "assets/videos/generic/landing-generic.mp4");
        assert_eq!(cfg.bg_video_path, "assets/videos/generic/landing-generic.mp4");
        assert_eq!(cfg.inactivity_video_path, "assets/videos/generic/promo-generic.mp4");
        assert_eq!(cfg.inactivity_video_path_2, "");
        assert_eq!(cfg.inactivity_video_path_3, "");
        assert_eq!(cfg.inactivity_timeout_sec, 120);
        assert_eq!(cfg.bg_base_color, "#cce5f8");
        assert_eq!(cfg.bg_start_color, "#c4e9f8");
        assert_eq!(cfg.bg_mid_color, "#cce5f8");
        assert_eq!(cfg.bg_end_color, "#dce2f8");
        assert_eq!(cfg.card_bg_color, "#F5F5F5");
        assert_eq!(cfg.card_border_color, "#bae6fd");
        assert_eq!(cfg.text_primary_color, "#0f172a");
        assert_eq!(cfg.text_secondary_color, "#475569");
        assert_eq!(cfg.btn_bg_color, "#0284c7");
        assert_eq!(cfg.btn_text_color, "#F5F5F5");
        assert!(!cfg.is_dark_mode);
        assert!(!cfg.setup_completed);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let mut cfg = AppConfig::default();
        cfg.price_card = "$599.990".to_string();
        cfg.sku = "SKU-990".to_string();
        cfg.setup_completed = true;

        let json = serde_json::to_string(&cfg).expect("Serialization failed");
        let deserialized: AppConfig = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.price_card, "$599.990");
        assert_eq!(deserialized.sku, "SKU-990");
        assert!(deserialized.setup_completed);
    }

    #[test]
    fn test_inactivity_video_list_slots() {
        let mut cfg = AppConfig::default();
        cfg.inactivity_video_path = "assets/videos/promo.mp4".to_string();
        cfg.inactivity_video_path_2 = "assets/videos/promo2.mp4".to_string();
        cfg.inactivity_video_path_3 = "".to_string();

        let list = cfg.get_inactivity_video_list();
        assert_eq!(list.len(), 2, "Only non-empty slots should be included");
        assert_eq!(list[0], "assets/videos/promo.mp4");
        assert_eq!(list[1], "assets/videos/promo2.mp4");
    }

    #[test]
    fn test_store_and_pricing_defaults() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.store_theme, "default");
        assert!(cfg.inactivity_timeout_sec >= 30, "Timeout should be at least 30s");
    }

    #[test]
    fn test_compute_default_video_paths() {
        // Case 1: ASUS + RTX
        let (central, inact_1, inact_2) = compute_default_video_paths("ASUS", "NVIDIA GeForce RTX 4060 Laptop GPU");
        assert_eq!(central, "assets/videos/generic/gaming_xbox_game_pass.mp4");
        assert_eq!(inact_1, "assets/videos/generic/promo-generic.mp4");
        assert_eq!(inact_2, "assets/videos/brand_a/v_a_p.mp4");

        // Case 2: ASUS + Non-RTX (e.g. Intel Iris Xe or AMD Radeon Graphics)
        let (central, inact_1, inact_2) = compute_default_video_paths("ASUS", "Intel Iris Xe Graphics");
        assert_eq!(central, "assets/videos/generic/landing-generic.mp4");
        assert_eq!(inact_1, "assets/videos/generic/promo-generic.mp4");
        assert_eq!(inact_2, "assets/videos/brand_a/v_a_p.mp4");

        // Case 3: Non-ASUS (e.g. HP) + RTX
        let (central, inact_1, inact_2) = compute_default_video_paths("HP", "NVIDIA GeForce RTX 3050 Laptop GPU");
        assert_eq!(central, "assets/videos/generic/gaming_xbox_game_pass.mp4");
        assert_eq!(inact_1, "assets/videos/generic/promo-generic.mp4");
        assert_eq!(inact_2, "");

        // Case 4: Non-ASUS (e.g. Lenovo) + Non-RTX
        let (central, inact_1, inact_2) = compute_default_video_paths("Lenovo", "AMD Radeon Graphics");
        assert_eq!(central, "assets/videos/generic/landing-generic.mp4");
        assert_eq!(inact_1, "assets/videos/generic/promo-generic.mp4");
        assert_eq!(inact_2, "");
    }
}
