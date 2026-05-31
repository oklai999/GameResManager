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

pub fn should_ignore_dir(path: &Path, settings: &ScanSettings) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let name_lower = name.to_ascii_lowercase();
    settings.ignored_directory_names.split(',').any(|ignored| {
        ignored.trim().eq_ignore_ascii_case(&name_lower)
    })
}

pub fn should_ignore_extension(ext: &str, settings: &ScanSettings) -> bool {
    let ext_normalized = ext.to_ascii_lowercase().trim_start_matches('.').to_string();
    settings.ignored_extensions.split(|c: char| c == ',' || c == '\n' || c == ' ').any(|ignored| {
        let trimmed = ignored.trim().trim_start_matches('.');
        !trimmed.is_empty() && trimmed.eq_ignore_ascii_case(&ext_normalized)
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
            thumbnail_cache_dir: None,
            database_path: None,
            ignored_extensions: "".to_string(),
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
            thumbnail_cache_dir: None,
            database_path: None,
            ignored_extensions: "".to_string(),
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
            thumbnail_cache_dir: None,
            database_path: None,
            ignored_extensions: "".to_string(),
updated_at: "2026-05-28T00:00:00Z".to_string(),
        };
        assert!(!asset_type_allowed("image", "png", &settings));
        assert!(asset_type_allowed("audio", "wav", &settings));
        assert!(asset_type_allowed("image", "psd", &settings));
    }

    #[test]
    fn ignores_configured_extensions() {
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
            thumbnail_cache_dir: None,
            database_path: None,
            ignored_extensions: "tmp, .log".to_string(),
updated_at: "2026-05-28T00:00:00Z".to_string(),
        };
        assert!(should_ignore_extension("tmp", &settings));
        assert!(should_ignore_extension("log", &settings));
        assert!(should_ignore_extension(".log", &settings));
        assert!(should_ignore_extension("TMP", &settings));
        assert!(!should_ignore_extension("png", &settings));
    }

    #[test]
    fn ignores_extensions_with_newlines_and_spaces() {
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
            thumbnail_cache_dir: None,
            database_path: None,
            ignored_extensions: "bak\n.tmp .old".to_string(),
updated_at: "2026-05-28T00:00:00Z".to_string(),
        };
        assert!(should_ignore_extension("bak", &settings));
        assert!(should_ignore_extension("tmp", &settings));
        assert!(should_ignore_extension("old", &settings));
        assert!(!should_ignore_extension("png", &settings));
    }
}
