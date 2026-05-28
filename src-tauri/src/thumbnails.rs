use anyhow::Context;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ThumbnailResult {
    pub thumbnail_path: String,
    pub width: i64,
    pub height: i64,
}

pub fn thumbnail_cache_path(cache_dir: &Path, source_path: &str, modified_at: &str) -> PathBuf {
    let mut hasher = Sha1::new();
    hasher.update(source_path.as_bytes());
    hasher.update(modified_at.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    cache_dir.join(format!("{hash}.webp"))
}

pub fn generate_image_thumbnail(
    source_path: &Path,
    source_key: &str,
    modified_at: &str,
    cache_dir: &Path,
) -> anyhow::Result<ThumbnailResult> {
    std::fs::create_dir_all(cache_dir).context("failed to create thumbnail cache directory")?;
    let image = image::open(source_path).context("failed to open image for thumbnail")?;
    let width = image.width() as i64;
    let height = image.height() as i64;
    let thumbnail = image.thumbnail(320, 320);
    let output_path = thumbnail_cache_path(cache_dir, source_key, modified_at);
    thumbnail
        .save_with_format(&output_path, image::ImageFormat::WebP)
        .context("failed to save thumbnail")?;

    Ok(ThumbnailResult {
        thumbnail_path: output_path.to_string_lossy().replace('\\', "/"),
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_path_is_stable_for_same_source_and_modified_time() {
        let first = thumbnail_cache_path(Path::new("cache"), "C:/a/icon.png", "2026-05-27");
        let second = thumbnail_cache_path(Path::new("cache"), "C:/a/icon.png", "2026-05-27");
        assert_eq!(first, second);
    }

    #[test]
    fn cache_path_changes_when_modified_time_changes() {
        let first = thumbnail_cache_path(Path::new("cache"), "C:/a/icon.png", "one");
        let second = thumbnail_cache_path(Path::new("cache"), "C:/a/icon.png", "two");
        assert_ne!(first, second);
    }
}
