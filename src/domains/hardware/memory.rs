use sysinfo::System;

pub fn get_ram_info(sys: &System, cpu_name: &str) -> (String, String) {
    let total_ram_bytes = sys.total_memory();
    let ram_gb = (total_ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u64;
    let ram = format!("{} GB", ram_gb);
    let ram_type = if cpu_name.contains("Ultra") || cpu_name.contains("8000") || cpu_name.contains("7000") {
        "DDR5"
    } else {
        "DDR4"
    }.to_string();
    (ram, ram_type)
}
