use sysinfo::Disks;

pub fn get_storage_info() -> String {
    let disks = Disks::new_with_refreshed_list();
    let mut total_disk_bytes: u64 = 0;
    for disk in &disks {
        if !disk.is_removable() {
            total_disk_bytes += disk.total_space();
        }
    }
    let total_gb = (total_disk_bytes as f64 / 1_000_000_000.0).round() as u64;
    if total_gb >= 900 {
        "1TB NVMe SSD".to_string()
    } else if total_gb >= 450 {
        "512GB SSD".to_string()
    } else if total_gb >= 220 {
        "256GB SSD".to_string()
    } else if total_gb >= 110 {
        "128GB SSD".to_string()
    } else {
        format!("{}GB SSD", total_gb)
    }
}
