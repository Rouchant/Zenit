pub fn resolve_brand_logo(brand: &str) -> Option<String> {
    let lower = brand.to_lowercase();
    if lower.contains("asus") {
        Some("assets/logos/asus.svg".to_string())
    } else if lower.contains("hp") {
        Some("assets/logos/hp.svg".to_string())
    } else if lower.contains("lenovo") {
        Some("assets/logos/lenovo.svg".to_string())
    } else if lower.contains("acer") {
        Some("assets/logos/acer.svg".to_string())
    } else if lower.contains("samsung") {
        Some("assets/logos/samsung.svg".to_string())
    } else {
        None
    }
}

pub fn resolve_store_logo(store_theme: &str) -> Option<String> {
    match store_theme.to_lowercase().as_str() {
        "falabella" => Some("assets/logos/falabella.svg".to_string()),
        "paris" => Some("assets/logos/paris.png".to_string()),
        "ripley" => Some("assets/logos/ripley.svg".to_string()),
        _ => None,
    }
}
