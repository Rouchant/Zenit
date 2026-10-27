pub mod battery;
pub mod connectivity;
pub mod cpu;
pub mod display;
pub mod gpu;
pub mod memory;
pub mod registry;
pub mod storage;
pub mod system_info;
pub mod types;

// Re-export core items for seamless backward compatibility
pub use battery::{detect_battery_capacity_wh, get_battery_info, parse_battery_wh_from_xml};
pub use connectivity::{
    detect_connectivity_features, format_connectivity_features, ConnectivityFeatures,
};
pub use cpu::{clean_processor_name, detect_generation, get_cpu_info, infer_processor_details};
pub use display::{approximate_refresh_rate, get_display_and_hz, get_display_raw_resolution, is_display_16_10};
pub use gpu::{get_gpu_name, rate_gpu};
pub use memory::get_ram_info;
pub use storage::{format_bytes_commercial, get_storage_info};
pub use system_info::{
    clean_brand_name, detect_os_badge, get_laptop_brand_and_model, get_os_info, is_oem_placeholder,
    refine_model_name,
};
pub use types::SystemSpecs;

use std::sync::RwLock;
use sysinfo::System;

static CACHED_SPECS: RwLock<Option<SystemSpecs>> = RwLock::new(None);

/// Retrieves system hardware specifications, returning cached results if available
/// to avoid repeating expensive PowerShell and powercfg subprocess calls.
pub fn get_hardware_specs() -> SystemSpecs {
    if let Ok(guard) = CACHED_SPECS.read() {
        if let Some(ref specs) = *guard {
            return specs.clone();
        }
    }

    let specs = detect_hardware();
    if let Ok(mut guard) = CACHED_SPECS.write() {
        *guard = Some(specs.clone());
    }
    specs
}

/// Forces a fresh detection of all system hardware and updates the global cache.
pub fn refresh_hardware_specs() -> SystemSpecs {
    let specs = detect_hardware();
    if let Ok(mut guard) = CACHED_SPECS.write() {
        *guard = Some(specs.clone());
    }
    specs
}

pub fn detect_hardware() -> SystemSpecs {
    let mut sys = System::new_all();
    sys.refresh_all();

    let (brand, model) = get_laptop_brand_and_model();
    let (display, refresh_rate) = get_display_and_hz();
    let gpu = get_gpu_name();
    let (battery, battery_percent, is_charging) = get_battery_info();
    let (cpu_name, cores, threads, vendor, generation) = get_cpu_info(&sys);
    let (ram, ram_type) = get_ram_info(&sys, &cpu_name);
    let storage = get_storage_info();
    let os = get_os_info();
    let connectivity = detect_connectivity_features();
    let others = format_connectivity_features(&connectivity);

    SystemSpecs {
        brand,
        model,
        processor: cpu_name,
        cores,
        threads,
        generation,
        vendor,
        ram,
        ram_type,
        storage,
        display,
        refresh_rate,
        gpu,
        os,
        battery,
        battery_percent,
        is_charging,
        others,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hardware_execution() {
        let specs = detect_hardware();
        assert!(!specs.processor.is_empty(), "Processor should not be empty");
        assert!(specs.cores > 0, "Cores should be at least 1");
        assert!(specs.threads > 0, "Threads should be at least 1");
        assert!(!specs.ram.is_empty(), "RAM should not be empty");
        assert!(!specs.os.is_empty(), "OS should not be empty");
    }

    #[test]
    fn test_ram_formatting() {
        let specs = detect_hardware();
        assert!(specs.ram.contains("GB"), "RAM description should contain GB");
    }

    #[test]
    fn test_brand_and_model_cleanup() {
        let specs = detect_hardware();
        assert!(
            !specs.brand.to_lowercase().contains("to be filled"),
            "Brand must not contain raw OEM placeholder"
        );
        assert!(
            !specs.model.to_lowercase().contains("to be filled"),
            "Model must not contain raw OEM placeholder"
        );
        assert!(!specs.brand.is_empty(), "Brand should not be empty");
        assert!(!specs.model.is_empty(), "Model should not be empty");
    }

    #[test]
    fn test_get_hardware_specs_caching() {
        let t1 = std::time::Instant::now();
        let _ = get_hardware_specs();
        let _elapsed1 = t1.elapsed();

        let t2 = std::time::Instant::now();
        let specs_cached = get_hardware_specs();
        let elapsed2 = t2.elapsed();

        // Cached call must be virtually instantaneous (sub-millisecond)
        assert!(elapsed2.as_millis() < 50, "Cached specs should return almost instantly");
        assert!(!specs_cached.processor.is_empty());
    }
}
