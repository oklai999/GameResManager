use crate::models::{AssetType, ScanSettings};
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

pub const THUMBNAIL_STATUS_NONE: &str = "none";

#[derive(Debug, Clone)]
pub struct ScannedAsset {
    pub absolute_path: String,
    pub file_name: String,
    pub extension: String,
    pub asset_type: String,
    pub file_size: i64,
    pub modified_at: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub thumbnail_path: Option<String>,
    pub thumbnail_status: String,
    pub thumbnail_error: Option<String>,
}

impl Default for ScannedAsset {
    fn default() -> Self {
        Self {
            absolute_path: String::new(),
            file_name: String::new(),
            extension: String::new(),
            asset_type: String::new(),
            file_size: 0,
            modified_at: String::new(),
            width: None,
            height: None,
            thumbnail_path: None,
            thumbnail_status: THUMBNAIL_STATUS_NONE.to_string(),
            thumbnail_error: None,
        }
    }
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
            ..Default::default()
        });
    }

    Ok(output)
}

pub fn should_ignore_dir(path: &Path, settings: &ScanSettings) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let name_lower = name.to_ascii_lowercase();
    settings.ignored_directory_names.split(',').any(|ignored| {
        ignored.trim().eq_ignore_ascii_case(&name_lower)
    })
}

pub fn asset_type_allowed(asset_type: &str, extension: &str, settings: &ScanSettings) -> bool {
    match asset_type {
        "image" if extension.eq_ignore_ascii_case("psd") => settings.include_psd,
        "image" => settings.include_images,
        "audio" => settings.include_audio,
        "video" => settings.include_video,
        "font" => settings.include_fonts,
        "model3d" => settings.include_models,
        "spine" => settings.include_spine,
        _ => true,
    }
}

pub fn should_generate_thumbnail(extension: &str, settings: &ScanSettings) -> bool {
    match extension.to_ascii_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" => true,
        "psd" => settings.generate_psd_thumbnails,
        _ => false,
    }
}

pub fn scan_folder_with_settings(path: &Path, settings: &ScanSettings) -> anyhow::Result<ScanOutput> {
    let mut output = ScanOutput::default();
    let mut it = WalkDir::new(path).follow_links(false).into_iter();

    while let Some(entry) = it.next() {
        let entry = match entry {
            Ok(value) => value,
            Err(_) => {
                output.skipped_count += 1;
                continue;
            }
        };

        if entry.file_type().is_dir() {
            if should_ignore_dir(entry.path(), settings) {
                it.skip_current_dir();
            }
            continue;
        }

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
        let extension = entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if asset_type == AssetType::Other || !asset_type_allowed(asset_type.as_str(), &extension, settings) {
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

        output.assets.push(ScannedAsset {
            absolute_path,
            file_name,
            extension,
            asset_type: asset_type.as_str().to_string(),
            file_size: metadata.len() as i64,
            modified_at: modified_at.to_rfc3339(),
            ..Default::default()
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

    #[test]
    fn ignores_configured_directories() {
        let settings = ScanSettings {
            id: 1,
            include_images: true,
            include_audio: true,
            include_video: true,
            include_fonts: true,
            include_models: true,
            include_spine: true,
            include_psd: true,
            generate_psd_thumbnails: false,
            ignored_directory_names: "node_modules,.git,target".to_string(),
            updated_at: "2026-05-28T00:00:00Z".to_string(),
        };
        assert!(should_ignore_dir(Path::new("/project/node_modules"), &settings));
        assert!(should_ignore_dir(Path::new("/project/.git"), &settings));
        assert!(should_ignore_dir(Path::new("/project/target"), &settings));
        assert!(!should_ignore_dir(Path::new("/project/assets"), &settings));
    }

    #[test]
    fn psd_thumbnail_generation_is_off_by_default() {
        let settings = ScanSettings {
            id: 1,
            include_images: true,
            include_audio: true,
            include_video: true,
            include_fonts: true,
            include_models: true,
            include_spine: true,
            include_psd: true,
            generate_psd_thumbnails: false,
            ignored_directory_names: "".to_string(),
            updated_at: "2026-05-28T00:00:00Z".to_string(),
        };
        assert!(!should_generate_thumbnail("psd", &settings));
        assert!(should_generate_thumbnail("png", &settings));
        assert!(should_generate_thumbnail("jpg", &settings));
    }

    #[test]
    fn disabled_image_scan_skips_png() {
        let settings = ScanSettings {
            id: 1,
            include_images: false,
            include_audio: true,
            include_video: true,
            include_fonts: true,
            include_models: true,
            include_spine: true,
            include_psd: true,
            generate_psd_thumbnails: false,
            ignored_directory_names: "".to_string(),
            updated_at: "2026-05-28T00:00:00Z".to_string(),
        };
        assert!(!asset_type_allowed("image", "png", &settings));
        assert!(asset_type_allowed("audio", "wav", &settings));
        assert!(asset_type_allowed("image", "psd", &settings));
    }
}
