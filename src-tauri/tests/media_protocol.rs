use game_resource_manager_lib::media;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;
use tempfile::TempDir;

async fn setup_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE library_folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL,
            last_scanned_at TEXT,
            is_enabled INTEGER NOT NULL DEFAULT 1
        )"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            library_folder_id INTEGER NOT NULL,
            absolute_path TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            extension TEXT NOT NULL,
            asset_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            modified_at TEXT NOT NULL,
            width INTEGER,
            height INTEGER,
            thumbnail_path TEXT,
            thumbnail_status TEXT NOT NULL DEFAULT 'none',
            thumbnail_error TEXT,
            note TEXT NOT NULL DEFAULT '',
            is_favorite INTEGER NOT NULL DEFAULT 0,
            is_missing INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (library_folder_id) REFERENCES library_folders(id) ON DELETE CASCADE
        )"
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn insert_folder(db: &SqlitePool, id: i64, path: &str) {
    sqlx::query(
        "INSERT INTO library_folders (id, name, path, created_at, is_enabled)
         VALUES (?1, 'test', ?2, '2024-01-01T00:00:00Z', 1)"
    )
    .bind(id)
    .bind(path)
    .execute(db)
    .await
    .unwrap();
}

async fn insert_asset(
    db: &SqlitePool,
    id: i64,
    absolute_path: &str,
    asset_type: &str,
    extension: &str,
    file_size: i64,
    is_missing: bool,
) {
    let file_name = Path::new(absolute_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    sqlx::query(
        "INSERT INTO assets (
            id, library_folder_id, absolute_path, file_name, extension, asset_type,
            file_size, modified_at, created_at, updated_at, is_missing
         ) VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', ?7)"
    )
    .bind(id)
    .bind(absolute_path)
    .bind(file_name)
    .bind(extension)
    .bind(asset_type)
    .bind(file_size)
    .bind(if is_missing { 1 } else { 0 })
    .execute(db)
    .await
    .unwrap();
}

async fn setup_db_with_asset(
    path: &Path,
    asset_type: &str,
    extension: &str,
    file_size: i64,
) -> SqlitePool {
    let db = setup_db().await;
    insert_folder(&db, 1, "I:\\test").await;
    insert_asset(&db, 1, &path.to_string_lossy(), asset_type, extension, file_size, false).await;
    db
}

#[test]
fn allows_only_selected_audio_and_video_extensions() {
    for (asset_type, ext) in [
        ("audio", "mp3"),
        ("audio", "wav"),
        ("audio", "ogg"),
        ("video", "mp4"),
        ("video", "webm"),
    ] {
        assert!(
            media::media_mime(asset_type, ext).is_some(),
            "expected {asset_type}/{ext} to be supported"
        );
    }
    assert!(media::media_mime("audio", "flac").is_none());
    assert!(media::media_mime("video", "mkv").is_none());
    assert!(media::media_mime("image", "mp4").is_none());

    // Case-insensitive matching.
    assert_eq!(media::media_mime("audio", "MP3"), Some("audio/mpeg"));
    assert_eq!(media::media_mime("video", "WEBM"), Some("video/webm"));
}

#[test]
fn parses_single_http_byte_range() {
    assert_eq!(
        media::parse_byte_range(Some("bytes=100-199"), 1000).unwrap(),
        Some((100, 199))
    );
    assert_eq!(
        media::parse_byte_range(Some("bytes=900-"), 1000).unwrap(),
        Some((900, 999))
    );
    assert!(media::parse_byte_range(Some("bytes=1000-1001"), 1000).is_err());
    assert!(media::parse_byte_range(Some("bytes=0-1,4-5"), 1000).is_err());
    assert!(media::parse_byte_range(Some("bytes=100-50"), 1000).is_err());
    assert!(media::parse_byte_range(Some("bytes=-"), 1000).is_err());
}

#[tokio::test]
async fn serves_only_the_indexed_asset_path() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("sound.mp3");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "audio", "mp3", 10).await;

    let response = media::build_media_response(&db, 1, None, false).await.unwrap();

    assert_eq!(response.status, 200);
    assert_eq!(response.mime, "audio/mpeg");
    assert_eq!(response.body, b"0123456789");
    assert_eq!(response.content_length, 10);
    assert!(response.content_range.is_none());
}

#[tokio::test]
async fn serves_partial_content_for_range_requests() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("movie.mp4");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "video", "mp4", 10).await;

    let response = media::build_media_response(&db, 1, Some("bytes=2-5"), false)
        .await
        .unwrap();
    assert_eq!(response.status, 206);
    assert_eq!(response.body, b"2345");
    assert_eq!(response.content_range.as_deref(), Some("bytes 2-5/10"));

    // Open-ended range is clamped to actual file length.
    let response = media::build_media_response(&db, 1, Some("bytes=8-"), false)
        .await
        .unwrap();
    assert_eq!(response.status, 206);
    assert_eq!(response.body, b"89");
    assert_eq!(response.content_range.as_deref(), Some("bytes 8-9/10"));
}

#[tokio::test]
async fn rejects_out_of_bounds_range() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("movie.mp4");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "video", "mp4", 10).await;

    let err = media::build_media_response(&db, 1, Some("bytes=10-15"), false)
        .await
        .unwrap_err();
    assert!(matches!(err, media::MediaError::RangeNotSatisfiable(_)));
}

#[tokio::test]
async fn rejects_multipart_range() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("movie.mp4");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "video", "mp4", 10).await;

    let err = media::build_media_response(&db, 1, Some("bytes=0-1,4-5"), false)
        .await
        .unwrap_err();
    assert!(matches!(err, media::MediaError::RangeNotSatisfiable(_)));
}

#[tokio::test]
async fn head_without_range_does_not_read_whole_file() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("big.mp4");
    let big = vec![b'x'; 64 * 1024 * 1024 + 1];
    std::fs::write(&path, &big).unwrap();
    let db = setup_db_with_asset(&path, "video", "mp4", big.len() as i64).await;

    let request = tauri::http::Request::builder()
        .method("HEAD")
        .uri("asset-media://localhost/1")
        .body(Vec::new())
        .unwrap();
    let response = media::handle_protocol_request(&db, request).await;
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.body().len(), 0);
    assert_eq!(
        response.headers().get("content-length").unwrap().to_str().unwrap(),
        big.len().to_string()
    );
}

#[tokio::test]
async fn head_request_returns_no_body() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("sound.mp3");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "audio", "mp3", 10).await;

    let request = tauri::http::Request::builder()
        .method("HEAD")
        .uri("asset-media://localhost/1")
        .body(Vec::new())
        .unwrap();
    let response = media::handle_protocol_request(&db, request).await;
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.body().len(), 0);
    assert_eq!(
        response.headers().get("content-length").unwrap().to_str().unwrap(),
        "10"
    );
}

#[tokio::test]
async fn rejects_unknown_missing_unsupported_and_unindexed_files() {
    let fixture = TempDir::new().unwrap();
    let missing_path = fixture.path().join("missing.mp3");
    let db = setup_db_with_asset(&missing_path, "audio", "mp3", 10).await;

    // Unknown ID.
    let err = media::build_media_response(&db, 999, None, false).await.unwrap_err();
    assert!(matches!(err, media::MediaError::NotFound));

    // File does not exist on disk.
    let err = media::build_media_response(&db, 1, None, false).await.unwrap_err();
    assert!(matches!(err, media::MediaError::NotFound));

    // Unsupported extension.
    let unsupported = fixture.path().join("sound.flac");
    std::fs::write(&unsupported, b"FLAC").unwrap();
    let unsupported_db =
        setup_db_with_asset(&unsupported, "audio", "flac", 4).await;
    let err = media::build_media_response(&unsupported_db, 1, None, false)
        .await
        .unwrap_err();
    assert!(matches!(err, media::MediaError::UnsupportedType));

    // Marked missing in the index.
    sqlx::query("UPDATE assets SET is_missing = 1 WHERE id = 1")
        .execute(&unsupported_db)
        .await
        .unwrap();
    let err = media::build_media_response(&unsupported_db, 1, None, false)
        .await
        .unwrap_err();
    assert!(matches!(err, media::MediaError::NotFound));
}

#[tokio::test]
async fn rejects_no_range_request_for_large_files() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("big.mp4");
    // Create a file larger than 64 MiB on disk, but keep the database file_size small
    // to prove we use filesystem metadata and not the database value.
    let big = vec![b'x'; 64 * 1024 * 1024 + 1];
    std::fs::write(&path, &big).unwrap();
    let db = setup_db_with_asset(&path, "video", "mp4", 1024).await;

    let err = media::build_media_response(&db, 1, None, false).await.unwrap_err();
    assert!(matches!(err, media::MediaError::PayloadTooLarge));

    // Range requests on the same large file remain allowed.
    let response = media::build_media_response(&db, 1, Some("bytes=0-1023"), false)
        .await
        .unwrap();
    assert_eq!(response.status, 206);
    assert_eq!(response.body.len(), 1024);
}

#[tokio::test]
async fn caps_single_range_to_max_bytes() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("chunked.mp3");
    let data = vec![b'a'; 2_000_000];
    std::fs::write(&path, &data).unwrap();
    let db = setup_db_with_asset(&path, "audio", "mp3", data.len() as i64).await;

    let response = media::build_media_response(&db, 1, Some("bytes=0-1999999"), false)
        .await
        .unwrap();
    assert_eq!(response.status, 206);
    assert_eq!(response.body.len(), 1_024_000);
    assert_eq!(
        response.content_range.as_deref(),
        Some("bytes 0-1023999/2000000")
    );
}

#[tokio::test]
async fn protocol_handler_rejects_invalid_paths() {
    let db = setup_db().await;

    for uri in [
        "asset-media://localhost/",
        "asset-media://localhost/abc",
        "asset-media://localhost/-1",
        "asset-media://localhost/1/extra",
    ] {
        let request = tauri::http::Request::builder()
            .uri(uri)
            .body(Vec::new())
            .unwrap();
        let response = media::handle_protocol_request(&db, request).await;
        assert_eq!(
            response.status().as_u16(),
            404,
            "expected 404 for {uri}"
        );
    }
}

#[tokio::test]
async fn uses_actual_metadata_length_not_database_file_size() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("mismatch.mp3");
    std::fs::write(&path, b"0123456789").unwrap();
    // Lie in the database about file_size.
    let db = setup_db_with_asset(&path, "audio", "mp3", 9999).await;

    let response = media::build_media_response(&db, 1, None, false).await.unwrap();
    assert_eq!(response.content_length, 10);
    assert_eq!(response.body, b"0123456789");
}

#[tokio::test]
async fn protocol_rejects_non_get_head_methods() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("sound.mp3");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "audio", "mp3", 10).await;

    for method in ["POST", "PUT", "DELETE", "OPTIONS", "PATCH"] {
        let request = tauri::http::Request::builder()
            .method(method)
            .uri("asset-media://localhost/1")
            .body(Vec::new())
            .unwrap();
        let response = media::handle_protocol_request(&db, request).await;
        assert_eq!(
            response.status().as_u16(),
            405,
            "expected 405 for {method}"
        );
    }
}

#[tokio::test]
async fn head_with_invalid_range_returns_416_with_content_range() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("movie.mp4");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "video", "mp4", 10).await;

    let request = tauri::http::Request::builder()
        .method("HEAD")
        .uri("asset-media://localhost/1")
        .header("Range", "bytes=10-15")
        .body(Vec::new())
        .unwrap();
    let response = media::handle_protocol_request(&db, request).await;
    assert_eq!(response.status().as_u16(), 416);
    assert_eq!(response.body().len(), 0);
    assert_eq!(
        response.headers().get("content-range").unwrap().to_str().unwrap(),
        "bytes */10"
    );
}

#[tokio::test]
async fn head_with_valid_range_returns_206_with_content_range() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("movie.mp4");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "video", "mp4", 10).await;

    let request = tauri::http::Request::builder()
        .method("HEAD")
        .uri("asset-media://localhost/1")
        .header("Range", "bytes=2-5")
        .body(Vec::new())
        .unwrap();
    let response = media::handle_protocol_request(&db, request).await;
    assert_eq!(response.status().as_u16(), 206);
    assert_eq!(response.body().len(), 0);
    assert_eq!(
        response.headers().get("content-range").unwrap().to_str().unwrap(),
        "bytes 2-5/10"
    );
    assert_eq!(
        response.headers().get("content-length").unwrap().to_str().unwrap(),
        "4"
    );
}

#[tokio::test]
async fn head_error_responses_have_no_body() {
    let db = setup_db().await;

    let request = tauri::http::Request::builder()
        .method("HEAD")
        .uri("asset-media://localhost/999")
        .body(Vec::new())
        .unwrap();
    let response = media::handle_protocol_request(&db, request).await;
    assert_eq!(response.status().as_u16(), 404);
    assert_eq!(response.body().len(), 0);
    assert_eq!(
        response.headers().get("content-length").unwrap().to_str().unwrap(),
        "0"
    );
}

#[tokio::test]
async fn get_416_includes_content_range_star() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("movie.mp4");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "video", "mp4", 10).await;

    let request = tauri::http::Request::builder()
        .uri("asset-media://localhost/1")
        .header("Range", "bytes=10-15")
        .body(Vec::new())
        .unwrap();
    let response = media::handle_protocol_request(&db, request).await;
    assert_eq!(response.status().as_u16(), 416);
    assert_eq!(response.body().len(), 0);
    assert_eq!(
        response.headers().get("content-range").unwrap().to_str().unwrap(),
        "bytes */10"
    );
}

#[tokio::test]
async fn protocol_rejects_plus_prefixed_asset_id() {
    let db = setup_db().await;

    let request = tauri::http::Request::builder()
        .uri("asset-media://localhost/+1")
        .body(Vec::new())
        .unwrap();
    let response = media::handle_protocol_request(&db, request).await;
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn rejects_path_extension_mismatch() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("sound.wav");
    std::fs::write(&path, b"0123456789").unwrap();
    // Database says it's an mp3, but the real file extension is wav.
    let db = setup_db_with_asset(&path, "audio", "mp3", 10).await;

    let err = media::build_media_response(&db, 1, None, false)
        .await
        .unwrap_err();
    assert!(matches!(err, media::MediaError::NotFound));
}

#[tokio::test]
async fn response_content_length_matches_actual_body() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("sound.mp3");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "audio", "mp3", 10).await;

    let response = media::build_media_response(&db, 1, None, false)
        .await
        .unwrap();
    assert_eq!(response.content_length, response.body.len() as u64);

    let response = media::build_media_response(
        &db, 1, Some("bytes=2-5"), false
    )
    .await
    .unwrap();
    assert_eq!(response.content_length, response.body.len() as u64);
}

#[tokio::test]
async fn protocol_forwards_range_header() {
    let fixture = TempDir::new().unwrap();
    let path = fixture.path().join("movie.mp4");
    std::fs::write(&path, b"0123456789").unwrap();
    let db = setup_db_with_asset(&path, "video", "mp4", 10).await;

    let request = tauri::http::Request::builder()
        .uri("asset-media://localhost/1")
        .header("Range", "bytes=2-5")
        .body(Vec::new())
        .unwrap();
    let response = media::handle_protocol_request(&db, request).await;
    assert_eq!(response.status().as_u16(), 206);
    assert_eq!(response.body(), b"2345");
    assert_eq!(
        response.headers().get("content-range").unwrap().to_str().unwrap(),
        "bytes 2-5/10"
    );
}
