//! Read-only, indexed-image preview. Never accepts filesystem paths from the UI.
use image::{ImageFormat, ImageReader};
use std::io::{Cursor, Read};
use tauri::http::{Request, Response};

const MAX_BYTES: u64 = 32 * 1024 * 1024;
static READ_SLOTS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
const MAX_PIXELS: u64 = 16_000_000;

fn read_image(path: &str) -> anyhow::Result<Vec<u8>> {
    let file = std::fs::File::open(path)?;
    anyhow::ensure!(file.metadata()?.is_file(), "不是普通图片文件");
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() as u64 <= MAX_BYTES, "图片超过 32 MB");
    let format = image::guess_format(&bytes)?;
    anyhow::ensure!(
        matches!(
            format,
            ImageFormat::Png
                | ImageFormat::Jpeg
                | ImageFormat::WebP
                | ImageFormat::Gif
                | ImageFormat::Bmp
        ),
        "不支持的图片格式"
    );
    let (w, h) = ImageReader::with_format(Cursor::new(&bytes), format).into_dimensions()?;
    anyhow::ensure!(
        u64::from(w) * u64::from(h) <= MAX_PIXELS,
        "图片超过 1600 万像素"
    );
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader.decode()?;
    let mut output = Cursor::new(Vec::new());
    decoded.write_to(&mut output, ImageFormat::Png)?;
    Ok(output.into_inner())
}

pub async fn handle_request(db: &crate::db::Db, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let result = async {
        anyhow::ensure!(request.method() == "GET", "unsupported method");
        let path = request.uri().path().strip_prefix('/').unwrap_or("");
        anyhow::ensure!(
            !path.is_empty() && path.bytes().all(|b| b.is_ascii_digit()),
            "invalid asset id"
        );
        let id: i64 = path.parse()?;
        anyhow::ensure!(id > 0, "invalid asset id");
        let asset = crate::db::get_asset_by_id(db, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("not found"))?;
        anyhow::ensure!(
            asset.asset_type == "image" && !asset.is_missing,
            "not an available image"
        );
        tokio::task::spawn_blocking(move || {
            let _permit = READ_SLOTS
                .try_acquire()
                .map_err(|_| anyhow::anyhow!("preview busy"))?;
            read_image(&asset.absolute_path)
        })
        .await?
    }
    .await;
    match result {
        Ok(bytes) => Response::builder()
            .header("Content-Type", "image/png")
            .header("Cache-Control", "no-store")
            .header("X-Content-Type-Options", "nosniff")
            .body(bytes)
            .unwrap(),
        Err(_) => Response::builder()
            .status(400)
            .header("Content-Type", "text/plain")
            .body(b"Image unavailable or exceeds preview limits".to_vec())
            .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn protocol_requires_existing_indexed_available_image() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::connect(&dir.path().join("fixture.sqlite"))
            .await
            .unwrap();
        let path = dir.path().join("sprite.png");
        image::RgbaImage::new(640, 32).save(&path).unwrap();
        let folder = crate::db::create_library_folder(&db, "fixture", dir.path().to_str().unwrap())
            .await
            .unwrap();
        sqlx::query("INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size, modified_at, created_at, updated_at) VALUES (1, ?, ?, 'sprite.png', 'png', 'image', 0, '', '', '')")
            .bind(folder.id).bind(path.to_str().unwrap()).execute(&db).await.unwrap();
        for uri in [
            "/0",
            "/2",
            "/../sprite.png",
            "/C:/sprite.png",
            "/+1",
            "/1/extra",
        ] {
            let response =
                handle_request(&db, Request::builder().uri(uri).body(vec![]).unwrap()).await;
            assert_ne!(response.status(), 200, "{uri}");
        }
        let response =
            handle_request(&db, Request::builder().uri("/1").body(vec![]).unwrap()).await;
        assert_eq!(response.status(), 200);
        assert_eq!(
            image::load_from_memory(response.body()).unwrap().width(),
            640
        );
        sqlx::query("UPDATE assets SET is_missing = 1 WHERE id = 1")
            .execute(&db)
            .await
            .unwrap();
        assert_ne!(
            handle_request(&db, Request::builder().uri("/1").body(vec![]).unwrap())
                .await
                .status(),
            200
        );
        db.close().await;
    }
    #[test]
    fn reads_original_dimensions_without_changing_source() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.png");
        image::RgbaImage::new(640, 480).save(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        let output = read_image(path.to_str().unwrap()).unwrap();
        assert_eq!(image::load_from_memory(&output).unwrap().width(), 640);
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
    #[test]
    fn rejects_oversized_and_non_image_files() {
        let file = tempfile::NamedTempFile::new().unwrap();
        file.as_file().set_len(MAX_BYTES + 1).unwrap();
        assert!(read_image(file.path().to_str().unwrap()).is_err());
        file.as_file().set_len(8).unwrap();
        assert!(read_image(file.path().to_str().unwrap()).is_err());
    }
}
