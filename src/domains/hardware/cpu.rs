use regex::Regex;
use std::sync::OnceLock;
use sysinfo::System;

static RE_INTEL: OnceLock<Regex> = OnceLock::new();
static RE_INTEL_CORE: OnceLock<Regex> = OnceLock::new();
static RE_RYZEN: OnceLock<Regex> = OnceLock::new();
static RE_N_SERIES: OnceLock<Regex> = OnceLock::new();

/// Limpia y normaliza el nombre del procesador eliminando ruido técnico
/// (marcas registradas, frecuencias, arquitecturas, sufijos WMI/Qualcomm).
pub fn clean_processor_name(raw: &str) -> String {
    let mut s = raw
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("(tm)", "")
        .replace("(r)", "")
        .trim()
        .to_string();

    while s.contains("  ") {
        s = s.replace("  ", " ");
    }

    // 1. Manejo específico para Snapdragon / Qualcomm
    if s.to_lowercase().contains("snapdragon") || s.to_lowercase().contains("qualcomm") {
        let cut_markers = [
            " - Qualcomm",
            " - Oryon",
            " - ARMv8",
            " Qualcomm Oryon",
            " Oryon CPU",
            " ARMv8",
        ];
        for marker in cut_markers {
            if let Some(pos) = s.find(marker) {
                s = s[..pos].trim().to_string();
                break;
            }
        }

        if s.starts_with("Qualcomm Snapdragon") {
            s = s.replacen("Qualcomm Snapdragon", "Snapdragon", 1);
        }

        if s.is_empty() || s == "Qualcomm" {
            s = "Snapdragon X".to_string();
        }

        return s.trim().to_string();
    }

    // 2. Manejo para Intel / AMD u otros procesadores
    if let Some(pos) = s.find(" CPU @") {
        s = s[..pos].trim().to_string();
    } else if let Some(pos) = s.find("@") {
        s = s[..pos].trim().to_string();
    }

    if s.to_lowercase().ends_with(" processor") {
        let lower = s.to_lowercase();
        if let Some(pos) = lower.rfind(" processor") {
            s = s[..pos].trim().to_string();
        }
    }

    let words: Vec<&str> = s.split_whitespace().collect();
    if let Some(last_word) = words.last() {
        if last_word.to_lowercase().ends_with("-core") {
            s = words[..words.len() - 1].join(" ");
        }
    }

    s.trim().to_string()
}

/// Identifica la generación del procesador mediante expresiones regulares aplicadas al nombre del modelo.
pub fn detect_generation(name: &str) -> String {
    let n = name.to_lowercase();
    let re_intel = RE_INTEL.get_or_init(|| Regex::new(r"i[3579]-(\d+)").unwrap());
    let re_intel_core = RE_INTEL_CORE.get_or_init(|| Regex::new(r"core\s+[3579]\s+(\d)").unwrap());
    let re_ryzen = RE_RYZEN.get_or_init(|| Regex::new(r"ryzen\s+[3579]\s+(\d)(\d{2,3})").unwrap());
    let re_n_series = RE_N_SERIES.get_or_init(|| Regex::new(r"n\d{3}").unwrap());

    if let Some(cap) = re_intel.captures(&n) {
        let digits = &cap[1];
        let gen_str = if digits.len() >= 5 {
            &digits[0..2]
        } else if digits.len() == 4 {
            // Para procesadores Intel de 10ª a 15ª Gen con modelos de 4 dígitos sin sufijo G (ej: i5-1235U, i5-1334U)
            if digits.starts_with('1') {
                &digits[0..2]
            } else {
                &digits[0..1]
            }
        } else {
            digits
        };
        format!("{}ª Gen", gen_str)
    } else if let Some(cap) = re_intel_core.captures(&n) {
        format!("Serie {}", &cap[1])
    } else if n.contains("ultra") {
        "Core Ultra".to_string()
    } else if n.contains("ryzen ai") {
        "Ryzen AI".to_string()
    } else if let Some(cap) = re_ryzen.captures(&n) {
        if cap[2].len() == 2 {
            format!("{}00 Series", &cap[1])
        } else {
            format!("{}000 Series", &cap[1])
        }
    } else if re_n_series.is_match(&n) {
        "N-Series".to_string()
    } else if n.contains("snapdragon") || n.contains("qualcomm") {
        if n.contains("snapdragon x")
            || n.contains("x elite")
            || n.contains("x plus")
            || n.contains("x1")
            || n.contains("x2")
            || n.contains("x3")
            || n.contains("x4")
        {
            "Snapdragon X".to_string()
        } else {
            "Qualcomm ARM".to_string()
        }
    } else {
        "Desconocida".to_string()
    }
}

pub fn infer_processor_details(name: &str) -> (String, String) {
    let clean = clean_processor_name(name);
    let lower = clean.to_lowercase();
    let vendor = if lower.contains("intel") {
        "Intel".to_string()
    } else if lower.contains("amd") {
        "AMD".to_string()
    } else if lower.contains("snapdragon") || lower.contains("qualcomm") {
        "Snapdragon".to_string()
    } else {
        "Generic".to_string()
    };

    let generation = detect_generation(&clean);
    (vendor, generation)
}

pub fn get_cpu_info(sys: &System) -> (String, usize, usize, String, String) {
    let raw_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .unwrap_or_else(|| "Procesador".to_string());
    let cpu_name = clean_processor_name(&raw_name);
    let cores = sys.physical_core_count().unwrap_or(sys.cpus().len());
    let threads = sys.cpus().len();
    let (vendor, generation) = infer_processor_details(&cpu_name);
    (cpu_name, cores, threads, vendor, generation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_generation_detection() {
        let cases = vec![
            ("Intel Core i5-1334U", "13ª Gen"),
            ("Intel Core i5-1335U", "13ª Gen"),
            ("Intel Core i5-1235U", "12ª Gen"),
            ("Intel Core i7-1355U", "13ª Gen"),
            ("Intel Core i7-12700H", "12ª Gen"),
            ("Intel Core i5-1135G7", "11ª Gen"),
            ("Intel Core i9-10900K", "10ª Gen"),
            ("Intel Core i7-9750H", "9ª Gen"),
            ("intel core i5-8250u", "8ª Gen"),
            ("Intel Core 5 120U", "Serie 1"),
            ("Intel Core 7 150U", "Serie 1"),
            ("intel core 3 100u", "Serie 1"),
            ("Intel Core Ultra 7 155H", "Core Ultra"),
            ("intel core ultra 9 185h", "Core Ultra"),
            ("Intel Processor N100", "N-Series"),
            ("Intel Processor N200", "N-Series"),
            ("intel processor n305", "N-Series"),
            ("AMD Ryzen AI 9 HX 370", "Ryzen AI"),
            ("amd ryzen ai 7 pro 360", "Ryzen AI"),
            ("AMD Ryzen 7 5700U", "5000 Series"),
            ("AMD Ryzen 5 5600X", "5000 Series"),
            ("AMD Ryzen 9 7900X", "7000 Series"),
            ("AMD Ryzen 5 7520U", "7000 Series"),
            ("amd ryzen 3 3250u", "3000 Series"),
            ("AMD Ryzen 7 270U", "200 Series"),
            ("AMD Ryzen 5 350U", "300 Series"),
            ("Snapdragon X - X126100", "Snapdragon X"),
            ("Snapdragon X Elite X1E-84-100", "Snapdragon X"),
            ("Qualcomm Snapdragon X Plus X1P-64-100", "Snapdragon X"),
            ("Qualcomm Snapdragon 8cx Gen 3", "Qualcomm ARM"),
            ("Intel Pentium Gold 7505", "Desconocida"),
        ];

        for (input, expected) in cases {
            assert_eq!(detect_generation(input), expected, "Fallo para CPU: {}", input);
        }
    }

    #[test]
    fn test_clean_processor_name() {
        assert_eq!(
            clean_processor_name("Snapdragon(R) X - X126100 - Qualcomm(R) Oryon(TM) CPU ARMv8 (64-bit) Family 8 Model 1 Revision 201 2956"),
            "Snapdragon X - X126100"
        );
        assert_eq!(
            clean_processor_name("Qualcomm(R) Snapdragon(R) X Plus X1P-64-100 - Qualcomm(R) Oryon(TM) CPU ARMv8 (64-bit)"),
            "Snapdragon X Plus X1P-64-100"
        );
        assert_eq!(
            clean_processor_name("Intel(R) Core(TM) i7-13700H CPU @ 2.40GHz"),
            "Intel Core i7-13700H"
        );
        assert_eq!(
            clean_processor_name("AMD Ryzen 5 5600X 6-Core Processor"),
            "AMD Ryzen 5 5600X"
        );
    }
}
