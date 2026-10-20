use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSpecs {
    pub brand: String,
    pub model: String,
    pub processor: String,
    pub cores: usize,
    pub threads: usize,
    #[serde(rename = "gen")]
    pub generation: String,
    pub vendor: String,
    pub ram: String,
    pub ram_type: String,
    pub storage: String,
    pub display: String,
    pub refresh_rate: u32,
    pub gpu: String,
    pub os: String,
    pub battery: String,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub others: String,
}
