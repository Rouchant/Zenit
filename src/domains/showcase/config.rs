use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

fn default_bg_start() -> String {
    "#f8fafc".to_string()
}
fn default_bg_end() -> String {
    "#e2e8f0".to_string()
}
fn default_card_bg() -> String {
    "#ffffff".to_string()
}
fn default_card_border() -> String {
    "#e2e8f0".to_string()
}
fn default_text_primary() -> String {
    "#0f172a".to_string()
}
fn default_text_secondary() -> String {
    "#64748b".to_string()
}
fn default_accent() -> String {
    "#0284c7".to_string()
}
fn default_btn_bg() -> String {
    "#0284c7".to_string()
}
fn default_btn_text() -> String {
    "#ffffff".to_string()
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

    #[serde(default = "default_bg_start")]
    pub bg_start_color: String,
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
            list.push("assets/videos/promo2.mp4".to_string());
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
            bg_gradient: "@linear-gradient(155deg, #f8fafc 0%, #e2e8f0 100%)".to_string(),
            store_theme: "default".to_string(),

            bg_start_color: "#f8fafc".to_string(),
            bg_end_color: "#e2e8f0".to_string(),
            card_bg_color: "#ffffff".to_string(),
            card_border_color: "#e2e8f0".to_string(),
            text_primary_color: "#0f172a".to_string(),
            text_secondary_color: "#64748b".to_string(),
            btn_bg_color: "#0284c7".to_string(),
            btn_text_color: "#ffffff".to_string(),

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

            central_video_path: "assets/videos/landing2.mp4".to_string(),
            bg_video_path: "assets/videos/bg.mp4".to_string(),
            inactivity_video_path: "assets/videos/promo2.mp4".to_string(),
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

fn load_config_from_disk() -> AppConfig {
    let path = get_config_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&content) {
                let mut modified = false;
                if cfg.central_video_path == "assets/videos/landing.mp4" || cfg.central_video_path.is_empty() {
                    cfg.central_video_path = "assets/videos/landing2.mp4".to_string();
                    modified = true;
                }
                if cfg.inactivity_video_path == "assets/videos/promo.mp4" {
                    cfg.inactivity_video_path = "assets/videos/promo2.mp4".to_string();
                    modified = true;
                }
                if cfg.bg_video_path == "assets/images/fallback-bg/bg.mp4" || cfg.bg_video_path.is_empty() {
                    cfg.bg_video_path = "assets/videos/bg.mp4".to_string();
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

                if modified {
                    let _ = save_config(&cfg);
                }
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
        assert_eq!(cfg.central_video_path, "assets/videos/landing2.mp4");
        assert_eq!(cfg.bg_video_path, "assets/videos/bg.mp4");
        assert_eq!(cfg.inactivity_video_path, "assets/videos/promo2.mp4");
        assert_eq!(cfg.inactivity_video_path_2, "");
        assert_eq!(cfg.inactivity_video_path_3, "");
        assert_eq!(cfg.inactivity_timeout_sec, 120);
        assert_eq!(cfg.bg_start_color, "#f8fafc");
        assert_eq!(cfg.card_bg_color, "#ffffff");
        assert_eq!(cfg.card_border_color, "#e2e8f0");
        assert_eq!(cfg.text_primary_color, "#0f172a");
        assert_eq!(cfg.text_secondary_color, "#64748b");
        assert_eq!(cfg.btn_bg_color, "#0284c7");
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
}
