use crate::models::AssetType;
use std::path::{Path, PathBuf};

pub fn classify_asset(path: &Path) -> AssetType {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" | "psd" => AssetType::Image,
        "mp3" | "wav" | "ogg" | "flac" => AssetType::Audio,
        "mp4" | "mov" | "webm" | "avi" => AssetType::Video,
        "ttf" | "otf" | "woff" | "woff2" => AssetType::Font,
        "gltf" | "glb" | "obj" | "fbx" => AssetType::Model3d,
        "skel" | "json" | "atlas" => AssetType::Spine,
        _ => AssetType::Other,
    }
}

pub fn normalize_path(path: &Path) -> anyhow::Result<String> {
    let absolute: PathBuf = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    Ok(absolute.to_string_lossy().replace('\\', "/"))
}

use chrono::{DateTime, Utc};
use std::fs;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ScannedAsset {
    pub absolute_path: String,
    pub file_name: String,
    pub extension: String,
    pub asset_type: String,
    pub file_size: i64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct ScanOutput {
    pub assets: Vec<ScannedAsset>,
    pub skipped_count: usize,
}

pub fn scan_folder(path: &Path) -> anyhow::Result<ScanOutput> {
    let mut output = ScanOutput::default();

    for entry in WalkDir::new(path).follow_links(false) {
        let entry = match entry {
            Ok(value) => value,
            Err(_) => {
                output.skipped_count += 1;
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let metadata = match fs::metadata(entry.path()) {
            Ok(value) => value,
            Err(_) => {
                output.skipped_count += 1;
                continue;
            }
        };

        let asset_type = classify_asset(entry.path());
        if asset_type == AssetType::Other {
            output.skipped_count += 1;
            continue;
        }

        let modified_at: DateTime<Utc> = metadata.modified()?.into();
        let absolute_path = normalize_path(entry.path())?;
        let file_name = entry
            .path()
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        let extension = entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        output.assets.push(ScannedAsset {
            absolute_path,
            file_name,
            extension,
            asset_type: asset_type.as_str().to_string(),
            file_size: metadata.len() as i64,
            modified_at: modified_at.to_rfc3339(),
        });
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_asset_types() {
        assert_eq!(classify_asset(Path::new("icon.PNG")), AssetType::Image);
        assert_eq!(classify_asset(Path::new("music.wav")), AssetType::Audio);
        assert_eq!(classify_asset(Path::new("cutscene.mp4")), AssetType::Video);
        assert_eq!(classify_asset(Path::new("font.ttf")), AssetType::Font);
        assert_eq!(classify_asset(Path::new("mesh.glb")), AssetType::Model3d);
        assert_eq!(classify_asset(Path::new("hero.skel")), AssetType::Spine);
    }

    #[test]
    fn unknown_extension_is_other() {
        assert_eq!(classify_asset(Path::new("readme.md")), AssetType::Other);
    }

    #[test]
    fn normalized_path_uses_forward_slashes() {
        let normalized = normalize_path(Path::new("folder\\asset.png")).unwrap();
        assert!(normalized.contains("folder/asset.png"));
    }
}
