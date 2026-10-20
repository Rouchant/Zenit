use sysinfo::System;

pub fn infer_processor_details(name: &str) -> (String, String) {
    let lower = name.to_lowercase();
    let vendor = if lower.contains("intel") {
        "Intel".to_string()
    } else if lower.contains("amd") {
        "AMD".to_string()
    } else {
        "Generic".to_string()
    };

    let mut generation = String::new();
    if lower.contains("ultra") {
        generation = "Core Ultra".to_string();
    } else if let Some(pos) = lower.find("i3-")
        .or_else(|| lower.find("i5-"))
        .or_else(|| lower.find("i7-"))
        .or_else(|| lower.find("i9-"))
    {
        let slice = &lower[pos + 3..];
        let num_str: String = slice.chars().take_while(|c| c.is_ascii_digit()).collect();
        if num_str.len() >= 2 {
            let gen_num = if num_str.len() >= 4 { &num_str[..2] } else { &num_str[..1] };
            generation = format!("{}ª Gen", gen_num);
        }
    } else if lower.contains("ryzen") {
        for series in &["7000", "8000", "5000", "6000", "4000", "3000"] {
            if lower.contains(series) {
                generation = format!("Serie {}", series);
                break;
            }
        }
    } else if lower.contains("n100") || lower.contains("n200") || lower.contains("n95") {
        generation = "N-Series".to_string();
    }

    (vendor, generation)
}

pub fn get_cpu_info(sys: &System) -> (String, usize, usize, String, String) {
    let cpu_name = sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_else(|| "Procesador".to_string());
    let cores = sys.physical_core_count().unwrap_or(sys.cpus().len());
    let threads = sys.cpus().len();
    let (vendor, generation) = infer_processor_details(&cpu_name);
    (cpu_name, cores, threads, vendor, generation)
}
