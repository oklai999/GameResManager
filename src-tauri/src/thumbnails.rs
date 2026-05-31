use anyhow::Context;
use image::ImageReader;
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
    let image = ImageReader::open(source_path)
        .context("failed to open image file")?
        .with_guessed_format()
        .context("failed to detect image format from content")?
        .decode()
        .context("failed to open image for thumbnail")?;
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

pub async fn generate_image_thumbnail_async(
    source_path: &std::path::Path,
    source_key: &str,
    modified_at: &str,
    cache_dir: &std::path::Path,
) -> anyhow::Result<ThumbnailResult> {
    let source_path = source_path.to_path_buf();
    let source_key = source_key.to_string();
    let modified_at = modified_at.to_string();
    let cache_dir = cache_dir.to_path_buf();

    tokio::task::spawn_blocking(move || {
        generate_image_thumbnail(&source_path, &source_key, &modified_at, &cache_dir)
    })
    .await
    .context("thumbnail generation task failed")?
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

    fn cleanup_test_files(tmp: &std::path::Path, paths: &[std::path::PathBuf]) {
        for p in paths {
            let _ = std::fs::remove_file(p);
        }
        let _ = std::fs::remove_dir(tmp);
    }

    #[test]
    fn generate_image_thumbnail_creates_webp_file() {
        let tmp = std::env::temp_dir().join(format!("grm_test_thumb_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let source_path = tmp.join("source.png");
        let img = image::RgbImage::new(20, 20);
        img.save(&source_path).unwrap();

        let result = generate_image_thumbnail(
            &source_path,
            source_path.to_str().unwrap(),
            "2026-05-27",
            &tmp,
        );

        assert!(result.is_ok(), "thumbnail generation failed: {:?}", result.err());
        let thumb = result.unwrap();
        assert!(thumb.thumbnail_path.ends_with(".webp"));
        assert!(std::path::Path::new(&thumb.thumbnail_path).exists());
        assert_eq!(thumb.width, 20);
        assert_eq!(thumb.height, 20);

        cleanup_test_files(&tmp, &[source_path, std::path::PathBuf::from(&thumb.thumbnail_path)]);
    }

    #[test]
    fn generate_image_thumbnail_resizes_large_image() {
        let tmp = std::env::temp_dir().join(format!("grm_test_thumb_resize_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let source_path = tmp.join("large.png");
        let img = image::RgbImage::new(800, 600);
        img.save(&source_path).unwrap();

        let result = generate_image_thumbnail(
            &source_path,
            source_path.to_str().unwrap(),
            "2026-05-27",
            &tmp,
        );

        assert!(result.is_ok());
        let thumb = result.unwrap();
        let thumb_img = image::open(&thumb.thumbnail_path).unwrap();
        assert!(thumb_img.width() <= 320);
        assert!(thumb_img.height() <= 320);

        cleanup_test_files(&tmp, &[source_path, std::path::PathBuf::from(&thumb.thumbnail_path)]);
    }
}
