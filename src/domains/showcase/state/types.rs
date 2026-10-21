use serde::{Deserialize, Serialize};

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
    pub bg_start: String,
    pub bg_end: String,
    pub card_bg: String,
    pub card_border: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub btn_bg: String,
    pub btn_text: String,
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
