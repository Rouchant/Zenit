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

pub fn list_vault_videos() -> Vec<VaultVideo> {
    let vault_dir = get_vault_dir();
    let mut videos = Vec::new();

    if let Ok(entries) = fs::read_dir(vault_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if ext == "mp4" || ext == "webm" || ext == "mkv" || ext == "mov" {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let size_mb = (size_bytes as f64) / (1024.0 * 1024.0);

                    // Original name without timestamp prefix if present
                    let name = if let Some(pos) = filename.find('_') {
                        filename[pos + 1..].replace('_', " ")
                    } else {
                        filename.clone()
                    };

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

pub fn delete_vault_video(filename: &str) -> Result<(), String> {
    let vault_dir = get_vault_dir();
    let target = vault_dir.join(filename);
    if target.exists() {
        fs::remove_file(target).map_err(|e| format!("Error al eliminar video: {}", e))?;
    }
    Ok(())
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
}
