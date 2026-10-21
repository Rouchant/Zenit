pub mod branding;
pub mod cards;
pub mod overrides;
pub mod types;

pub use branding::{resolve_brand_logo, resolve_store_logo};
pub use cards::{build_left_specs, build_right_specs};
pub use overrides::apply_config_overrides;
pub use types::{DisplaySpecCard, ThemeColors, ZenitUiState};

use super::config::load_config;
use crate::domains::hardware::get_hardware_specs;

pub fn build_ui_state() -> ZenitUiState {
    let mut specs = get_hardware_specs();
    let cfg = load_config();

    // 1. Apply overrides
    apply_config_overrides(&mut specs, &cfg);

    // 2. Logos & Branding
    let brand_logo = resolve_brand_logo(&specs.brand);
    let store_logo = resolve_store_logo(&cfg.store_theme);

    // 3. Build Column Cards
    let left_specs = build_left_specs(&specs);
    let right_specs = build_right_specs(&specs, &cfg);

    // 4. Pricing & SKU
    let price_card = if cfg.price_card.trim().is_empty() {
        None
    } else {
        Some(cfg.price_card.trim().to_string())
    };
    let price_all = if cfg.price_all_methods.trim().is_empty() {
        None
    } else {
        Some(cfg.price_all_methods.trim().to_string())
    };
    let sku = if cfg.sku.trim().is_empty() {
        None
    } else {
        Some(cfg.sku.trim().to_string())
    };
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
            bg_start: cfg.bg_start_color,
            bg_end: cfg.bg_end_color,
            card_bg: cfg.card_bg_color,
            card_border: cfg.card_border_color,
            text_primary: cfg.text_primary_color,
            text_secondary: cfg.text_secondary_color,
            btn_bg: cfg.btn_bg_color,
            btn_text: cfg.btn_text_color,
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
        assert!(!state.brand_name.is_empty(), "Brand name should not be empty");
        assert_eq!(state.left_specs.len(), 4, "Left column must have 4 core specs cards");
        assert!(state.right_specs.len() >= 4, "Right column must have at least Display, GPU, Battery, Model");
        assert!(!state.center_video.is_empty(), "Center video path should not be empty");
    }

    #[test]
    fn test_colors_initialization() {
        let state = build_ui_state();
        assert!(state.colors.primary.starts_with('#'));
        assert!(state.colors.secondary.starts_with('#'));
        assert!(state.colors.accent.starts_with('#'));
        assert!(!state.colors.bg_gradient.is_empty());
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
        assert_eq!(battery_card.label, "Batería");
        assert!(!model_card.value.is_empty());
        assert!(!battery_card.value.is_empty());
        assert!(battery_card.value.to_lowercase().contains("hora") || battery_card.value.to_lowercase().contains("wh"));

        if let Some(others_card) = state.right_specs.iter().find(|s| s.id == "card-others") {
            assert_eq!(others_card.label, "Otros y Conectividad");
        }
    }
}
