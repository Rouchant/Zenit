use sysinfo::Disks;

/// Formatea los bytes totales para representarlos con capacidades comerciales de Retail (SSD/almacenamiento).
pub fn format_bytes_commercial(total_bytes: u64) -> String {
    let total_gb = total_bytes as f64 / 1_000_000_000.0;

    if total_gb >= 872.0 {
        let tb = total_gb / 1000.0;
        let fract = tb.fract();
        if fract >= 0.872 {
            format!("{:.0}TB SSD", tb.ceil())
        } else if fract <= 0.128 {
            format!("{:.0}TB SSD", tb.floor())
        } else {
            format!("{:.1}TB SSD", tb).replace(".", ",")
        }
    } else {
        let rounded = (total_gb / 128.0).round() * 128.0;
        format!("{:.0}GB SSD", if rounded == 0.0 { total_gb.round() } else { rounded })
    }
}

pub fn get_storage_info() -> String {
    let disks = Disks::new_with_refreshed_list();
    let mut total_disk_bytes: u64 = 0;
    for disk in &disks {
        if !disk.is_removable() {
            total_disk_bytes += disk.total_space();
        }
    }

    if total_disk_bytes > 0 {
        format_bytes_commercial(total_disk_bytes)
    } else {
        "512GB SSD".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_commercial() {
        // Tamaños estándar
        assert_eq!(format_bytes_commercial(512_000_000_000), "512GB SSD");
        assert_eq!(format_bytes_commercial(1_024_000_000_000), "1TB SSD");

        // Redondeo comercial (lo que Windows lee vs lo que se vende en retail)
        assert_eq!(format_bytes_commercial(476_940_000_000), "512GB SSD"); // 512GB real
        assert_eq!(format_bytes_commercial(931_000_000_000), "1TB SSD");   // 1TB real
        assert_eq!(format_bytes_commercial(238_000_000_000), "256GB SSD"); // 256GB real
        assert_eq!(format_bytes_commercial(119_000_000_000), "128GB SSD"); // 128GB real
    }
}
