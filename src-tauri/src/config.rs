use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Theme & Colors
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub bg_gradient: String,
    pub store_theme: String, // "default", "falabella", "paris", "ripley"

    // Optional Pricing & SKU
    pub price_card: String,
    pub price_all_methods: String,
    pub sku: String,

    // Custom Spec Overrides
    pub brand_override: Option<String>,
    pub model_override: Option<String>,
    pub processor_override: Option<String>,
    pub ram_override: Option<String>,
    pub storage_override: Option<String>,
    pub display_override: Option<String>,
    pub gpu_override: Option<String>,
    pub os_override: Option<String>,
    pub battery_override: Option<String>,
    pub other_specs: String, // Campo "Otros" personalizable

    // Videos
    pub central_video_path: String,
    pub bg_video_path: String,
    pub inactivity_video_path: String,
    pub inactivity_timeout_sec: u64,

    // First Run Setup
    #[serde(default)]
    pub setup_completed: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            primary_color: "#00f2ff".to_string(),
            secondary_color: "#0072ff".to_string(),
            accent_color: "#00e5ff".to_string(),
            bg_gradient: "radial-gradient(circle at center, #111422 0%, #06070a 100%)".to_string(),
            store_theme: "default".to_string(),

            price_card: "".to_string(),
            price_all_methods: "".to_string(),
            sku: "".to_string(),

            brand_override: None,
            model_override: None,
            processor_override: None,
            ram_override: None,
            storage_override: None,
            display_override: None,
            gpu_override: None,
            os_override: None,
            battery_override: None,
            other_specs: "Wi-Fi 6 / Bluetooth 5.3 / Teclado en Español".to_string(),

            central_video_path: "assets/videos/landing.mp4".to_string(),
            bg_video_path: "assets/images/fallback-bg/bg.mp4".to_string(),
            inactivity_video_path: "assets/videos/promo.mp4".to_string(),
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

pub fn load_config() -> AppConfig {
    let path = get_config_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }
    }
    let default_cfg = AppConfig::default();
    let _ = save_config(&default_cfg);
    default_cfg
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_file_path();
    let data = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Error al serializar config: {}", e))?;
    fs::write(path, data)
        .map_err(|e| format!("Error al guardar config en disco: {}", e))?;
    Ok(())
}

pub fn verify_password(input: &str) -> bool {
    let clean = input.trim().to_lowercase();
    clean == "demo" || clean == "z3n1t"
}
