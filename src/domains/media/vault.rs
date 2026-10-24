use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultVideo {
    pub name: String,
    pub filename: String,
    pub path: String,
    pub size_mb: f64,
}

pub fn get_vault_dir() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let vault_dir = Path::new(&app_data).join("Zenit").join("vault");
    if !vault_dir.exists() {
        let _ = fs::create_dir_all(&vault_dir);
    }
    vault_dir
}

pub fn import_video_to_vault(source_path: &str) -> Result<VaultVideo, String> {
    let src = Path::new(source_path);
    if !src.exists() {
        return Err("El archivo de origen no existe".to_string());
    }

    let vault_dir = get_vault_dir();
    let original_name = src.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4");

    // Generate unique timestamp prefix
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let safe_filename = format!("{}_{}", timestamp, original_name.replace(' ', "_"));
    let dest_path = vault_dir.join(&safe_filename);

    fs::copy(src, &dest_path)
        .map_err(|e| format!("Error al copiar el video al baúl: {}", e))?;

    let size_bytes = fs::metadata(&dest_path).map(|m| m.len()).unwrap_or(0);
    let size_mb = (size_bytes as f64) / (1024.0 * 1024.0);

    Ok(VaultVideo {
        name: original_name.to_string(),
        filename: safe_filename,
        path: dest_path.to_string_lossy().to_string(),
        size_mb: (size_mb * 10.0).round() / 10.0,
    })
}

pub const GENERIC_VIDEOS: &[(&str, &str, &str)] = &[
    ("assets/videos/generic/landing-generic.mp4", "landing-generic.mp4", "Landing Genérico"),
    ("assets/videos/generic/promo-generic.mp4", "promo-generic.mp4", "Promo Genérico"),
    ("assets/videos/generic/gaming_xbox_game_pass.mp4", "gaming_xbox_game_pass.mp4", "Xbox Gamepass"),
    ("assets/videos/generic/windows_the_home_of_gaming.mp4", "windows_the_home_of_gaming.mp4", "Windows House of Gaming"),
];

pub const ASUS_VIDEOS: &[(&str, &str, &str)] = &[
    ("assets/videos/brand_a/v_a_cer.mp4", "v_a_cer.mp4", "Ceraluminum"),
    ("assets/videos/brand_a/v_a_gp.mp4", "v_a_gp.mp4", "Garantia Perfecta Asus"),
    ("assets/videos/brand_a/v_a_l.mp4", "v_a_l.mp4", "AI Asus"),
    ("assets/videos/brand_a/v_a_oled.mp4", "v_a_oled.mp4", "Asus OLED"),
    ("assets/videos/brand_a/v_a_p.mp4", "v_a_p.mp4", "Asus Grado militar"),
    ("assets/videos/brand_a/v_a_qd.mp4", "v_a_qd.mp4", "Asus Rog Quality"),
    ("assets/videos/brand_a/v_a_tuf.mp4", "v_a_tuf.mp4", "TUF Quality"),
    ("assets/videos/brand_a/v_a_vw.mp4", "v_a_vw.mp4", "Vivobook"),
    ("assets/videos/brand_a/v_a_zb26.mp4", "v_a_zb26.mp4", "Asus Zenbook 2026"),
];

/// Resuelve la ruta absoluta y tamaño de un archivo de video en el sistema de assets.
fn resolve_video_asset(rel_path: &str) -> Option<(PathBuf, f64)> {
    let p = Path::new(rel_path);
    let target = if p.exists() {
        Some(p.to_path_buf())
    } else if let Ok(cur) = std::env::current_dir() {
        let full = cur.join(rel_path);
        if full.exists() {
            Some(full)
        } else {
            None
        }
    } else {
        None
    };

    let final_path = target.or_else(|| {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let full = parent.join(rel_path);
                if full.exists() {
                    return Some(full);
                }
            }
        }
        None
    })?;

    let size_bytes = fs::metadata(&final_path).map(|m| m.len()).unwrap_or(0);
    let size_mb = (size_bytes as f64) / (1024.0 * 1024.0);
    Some((final_path, (size_mb * 10.0).round() / 10.0))
}

/// Devuelve el título simplificado asignado a un video reconocido.
pub fn get_simplified_title(filename: &str) -> Option<&'static str> {
    let lower = filename.to_lowercase();
    if lower.contains("v_a_cer") {
        Some("Ceraluminum")
    } else if lower.contains("v_a_gp") {
        Some("Garantia Perfecta Asus")
    } else if lower.contains("v_a_oled") {
        Some("Asus OLED")
    } else if lower.contains("v_a_l") {
        Some("AI Asus")
    } else if lower.contains("v_a_p") {
        Some("Asus Grado militar")
    } else if lower.contains("v_a_qd") {
        Some("Asus Rog Quality")
    } else if lower.contains("v_a_tuf") {
        Some("TUF Quality")
    } else if lower.contains("v_a_vw") {
        Some("Vivobook")
    } else if lower.contains("v_a_zb26") {
        Some("Asus Zenbook 2026")
    } else if lower.contains("gaming_xbox_game_pass") {
        Some("Xbox Gamepass")
    } else if lower.contains("windows_the_home_of_gaming") {
        Some("Windows House of Gaming")
    } else if lower.contains("landing-generic") {
        Some("Landing Genérico")
    } else if lower.contains("promo-generic") {
        Some("Promo Genérico")
    } else {
        None
    }
}

/// Comprueba si la marca configurada o detectada en la máquina es ASUS.
pub fn is_brand_asus() -> bool {
    let cfg = crate::domains::showcase::config::load_config();
    if let Some(ref bo) = cfg.brand_override {
        if !bo.trim().is_empty() {
            return bo.to_lowercase().contains("asus");
        }
    }
    let specs = crate::domains::hardware::get_hardware_specs();
    specs.brand.to_lowercase().contains("asus")
}

pub fn list_vault_videos() -> Vec<VaultVideo> {
    list_vault_videos_for_brand(is_brand_asus())
}

pub fn list_vault_videos_for_brand(is_asus: bool) -> Vec<VaultVideo> {
    let mut videos = Vec::new();
    let mut seen_filenames = std::collections::HashSet::new();

    // 1. Videos genéricos precargados (siempre disponibles)
    for &(rel_path, filename, title) in GENERIC_VIDEOS {
        if let Some((path, size_mb)) = resolve_video_asset(rel_path) {
            seen_filenames.insert(filename.to_string());
            videos.push(VaultVideo {
                name: title.to_string(),
                filename: filename.to_string(),
                path: path.to_string_lossy().to_string(),
                size_mb,
            });
        }
    }

    // 2. Videos exclusivos ASUS brand_a (SOLO si la marca es ASUS)
    if is_asus {
        for &(rel_path, filename, title) in ASUS_VIDEOS {
            if let Some((path, size_mb)) = resolve_video_asset(rel_path) {
                seen_filenames.insert(filename.to_string());
                videos.push(VaultVideo {
                    name: title.to_string(),
                    filename: filename.to_string(),
                    path: path.to_string_lossy().to_string(),
                    size_mb,
                });
            }
        }
    }

    // 3. Videos importados por el usuario desde el baúl seguro (%APPDATA%\Zenit\vault)
    let vault_dir = get_vault_dir();
    if let Ok(entries) = fs::read_dir(vault_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if ext == "mp4" || ext == "webm" || ext == "mkv" || ext == "mov" {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    if seen_filenames.contains(&filename) {
                        continue;
                    }

                    // Si es un video de ASUS y la máquina actual NO es ASUS, no mostrarlo
                    let is_asus_video = ASUS_VIDEOS.iter().any(|(_, f, _)| *f == filename || filename.contains(f));
                    if is_asus_video && !is_asus {
                        continue;
                    }

                    let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let size_mb = (size_bytes as f64) / (1024.0 * 1024.0);

                    // Formatear título simplificado
                    let name = if let Some(title) = get_simplified_title(&filename) {
                        title.to_string()
                    } else if let Some(pos) = filename.find('_') {
                        filename[pos + 1..].replace('_', " ")
                    } else {
                        filename.clone()
                    };

                    seen_filenames.insert(filename.clone());
                    videos.push(VaultVideo {
                        name,
                        filename,
                        path: path.to_string_lossy().to_string(),
                        size_mb: (size_mb * 10.0).round() / 10.0,
                    });
                }
            }
        }
    }

    videos
}

pub fn is_preloaded_video(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    GENERIC_VIDEOS.iter().any(|(_, f, _)| lower == f.to_lowercase() || lower.ends_with(&f.to_lowercase()))
        || ASUS_VIDEOS.iter().any(|(_, f, _)| lower == f.to_lowercase() || lower.ends_with(&f.to_lowercase()))
}

pub fn delete_vault_video(filename: &str) -> Result<(), String> {
    if is_preloaded_video(filename) {
        return Err("No se pueden borrar los videos precargados del sistema".to_string());
    }
    let vault_dir = get_vault_dir();
    let target = vault_dir.join(filename);
    if target.exists() {
        fs::remove_file(target).map_err(|e| format!("Error al eliminar video: {}", e))?;
    }
    Ok(())
}

use std::sync::atomic::{AtomicBool, Ordering};

static VAULT_SEEDED: AtomicBool = AtomicBool::new(false);

pub fn seed_vault_defaults() {
    if VAULT_SEEDED.swap(true, Ordering::SeqCst) {
        return;
    }
    // Asegura que la carpeta del baúl exista para guardar videos importados por el usuario
    let _ = get_vault_dir();
}

pub fn pick_video_file() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dlg = New-Object System.Windows.Forms.OpenFileDialog
$dlg.Filter = "Videos (*.mp4, *.webm, *.mkv, *.mov)|*.mp4;*.webm;*.mkv;*.mov"
$dlg.Title = "Seleccionar video para el Baúl Seguro"
if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dlg.FileName
}
"#;
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .ok()?;
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

pub fn pick_image_file() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dlg = New-Object System.Windows.Forms.OpenFileDialog
$dlg.Filter = "Imágenes y Logos (*.svg;*.png;*.jpg;*.jpeg;*.webp)|*.svg;*.png;*.jpg;*.jpeg;*.webp"
$dlg.Title = "Seleccionar Logo de Marca"
if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dlg.FileName
}
"#;
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .ok()?;
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_dir_creation() {
        let vault_dir = get_vault_dir();
        assert!(vault_dir.exists(), "Vault directory must exist or be created");
    }

    #[test]
    fn test_list_vault_videos_runs_without_panic() {
        let list = list_vault_videos();
        // Simply assert list can be iterated over
        for v in list {
            assert!(!v.filename.is_empty());
            assert!(!v.path.is_empty());
        }
    }

    #[test]
    fn test_simplified_titles_mapping() {
        assert_eq!(get_simplified_title("v_a_cer.mp4"), Some("Ceraluminum"));
        assert_eq!(get_simplified_title("v_a_gp.mp4"), Some("Garantia Perfecta Asus"));
        assert_eq!(get_simplified_title("v_a_l.mp4"), Some("AI Asus"));
        assert_eq!(get_simplified_title("v_a_oled.mp4"), Some("Asus OLED"));
        assert_eq!(get_simplified_title("v_a_p.mp4"), Some("Asus Grado militar"));
        assert_eq!(get_simplified_title("v_a_qd.mp4"), Some("Asus Rog Quality"));
        assert_eq!(get_simplified_title("v_a_tuf.mp4"), Some("TUF Quality"));
        assert_eq!(get_simplified_title("v_a_vw.mp4"), Some("Vivobook"));
        assert_eq!(get_simplified_title("v_a_zb26.mp4"), Some("Asus Zenbook 2026"));
        assert_eq!(get_simplified_title("gaming_xbox_game_pass.mp4"), Some("Xbox Gamepass"));
        assert_eq!(get_simplified_title("windows_the_home_of_gaming.mp4"), Some("Windows House of Gaming"));
        assert_eq!(get_simplified_title("landing-generic.mp4"), Some("Landing Genérico"));
        assert_eq!(get_simplified_title("promo-generic.mp4"), Some("Promo Genérico"));
    }

    #[test]
    fn test_asus_exclusivity_filtering() {
        let generic_only = list_vault_videos_for_brand(false);
        for v in &generic_only {
            assert!(!v.filename.starts_with("v_a_"), "Non-ASUS vault must not contain ASUS exclusive videos: {}", v.filename);
        }

        let with_asus = list_vault_videos_for_brand(true);
        let has_asus = with_asus.iter().any(|v| v.filename.starts_with("v_a_"));
        assert!(has_asus, "ASUS vault must contain ASUS exclusive videos");
    }
}
