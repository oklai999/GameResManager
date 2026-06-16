use std::io::{Read, Seek, SeekFrom};

use crate::db;
use crate::db::Db;

const MAX_RANGE_BYTES: u64 = 1_024_000;
const MAX_WHOLE_FILE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("asset not found")]
    NotFound,
    #[error("unsupported media type")]
    UnsupportedType,
    #[error("range not satisfiable")]
    RangeNotSatisfiable(Option<u64>),
    #[error("payload too large")]
    PayloadTooLarge,
    #[error("database error: {0}")]
    Database(#[from] anyhow::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaResponse {
    pub status: u16,
    pub mime: String,
    pub content_length: u64,
    pub content_range: Option<String>,
    pub body: Vec<u8>,
}

pub fn media_mime(asset_type: &str, extension: &str) -> Option<&'static str> {
    match (asset_type, extension.to_ascii_lowercase().as_str()) {
        ("audio", "mp3") => Some("audio/mpeg"),
        ("audio", "wav") => Some("audio/wav"),
        ("audio", "ogg") => Some("audio/ogg"),
        ("video", "mp4") => Some("video/mp4"),
        ("video", "webm") => Some("video/webm"),
        _ => None,
    }
}

pub fn parse_byte_range(range: Option<&str>, len: u64) -> Result<Option<(u64, u64)>, MediaError> {
    let range = match range {
        Some(r) => r.trim(),
        None => return Ok(None),
    };

    let suffix = range
        .strip_prefix("bytes=")
        .ok_or(MediaError::RangeNotSatisfiable(None))?;

    // Only single-part ranges are supported.
    if suffix.contains(',') {
        return Err(MediaError::RangeNotSatisfiable(Some(len)));
    }

    let (start_str, end_str) = suffix
        .split_once('-')
        .ok_or(MediaError::RangeNotSatisfiable(Some(len)))?;

    let start = if start_str.is_empty() {
        return Err(MediaError::RangeNotSatisfiable(Some(len)));
    } else {
        start_str
            .parse::<u64>()
            .map_err(|_| MediaError::RangeNotSatisfiable(Some(len)))?
    };

    let end = if end_str.is_empty() {
        len.saturating_sub(1)
    } else {
        end_str
            .parse::<u64>()
            .map_err(|_| MediaError::RangeNotSatisfiable(Some(len)))?
    };

    if start >= len || end >= len || end < start {
        return Err(MediaError::RangeNotSatisfiable(Some(len)));
    }

    Ok(Some((start, end)))
}

pub async fn build_media_response(
    db: &Db,
    asset_id: i64,
    range: Option<&str>,
    head_only: bool,
) -> Result<MediaResponse, MediaError> {
    let asset = db::get_asset_by_id(db, asset_id)
        .await?
        .ok_or(MediaError::NotFound)?;

    if asset.is_missing {
        return Err(MediaError::NotFound);
    }

    let mime = media_mime(&asset.asset_type, &asset.extension)
        .ok_or(MediaError::UnsupportedType)?
        .to_string();

    let path = std::path::Path::new(&asset.absolute_path).to_path_buf();
    let db_extension = asset.extension.to_ascii_lowercase();

    // The indexed path extension must match the recorded extension.
    let actual_extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if actual_extension != db_extension {
        return Err(MediaError::NotFound);
    }

    let range = range.map(|s| s.to_string());

    tokio::task::spawn_blocking(move || {
        let mut file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(MediaError::NotFound);
            }
            Err(e) => return Err(MediaError::Io(e)),
        };
        let metadata = file.metadata()?;
        let len = metadata.len();

        let parsed_range = parse_byte_range(range.as_deref(), len)?;

        if head_only {
            // HEAD must not read file contents. Validate the range and return headers only.
            if let Some((start, end)) = parsed_range {
                let requested_len = end - start + 1;
                let content_length = requested_len.min(MAX_RANGE_BYTES);
                let actual_end = start + content_length - 1;
                return Ok(MediaResponse {
                    status: 206,
                    mime,
                    content_length,
                    content_range: Some(format!("bytes {start}-{actual_end}/{len}")),
                    body: Vec::new(),
                });
            }
            return Ok(MediaResponse {
                status: 200,
                mime,
                content_length: len,
                content_range: None,
                body: Vec::new(),
            });
        }

        if let Some((start, end)) = parsed_range {
            file.seek(SeekFrom::Start(start))?;
            let requested_len = end - start + 1;
            let bytes_to_read = requested_len.min(MAX_RANGE_BYTES);
            let mut buf = Vec::with_capacity(bytes_to_read as usize);
            file.take(bytes_to_read).read_to_end(&mut buf)?;
            if buf.is_empty() {
                // File shrank after metadata; range is no longer satisfiable.
                return Err(MediaError::RangeNotSatisfiable(Some(0)));
            }
            let actual_end = start + buf.len() as u64 - 1;
            return Ok(MediaResponse {
                status: 206,
                mime,
                content_length: buf.len() as u64,
                content_range: Some(format!("bytes {start}-{actual_end}/{len}")),
                body: buf,
            });
        }

        if len > MAX_WHOLE_FILE_BYTES {
            return Err(MediaError::PayloadTooLarge);
        }

        let mut buf = Vec::with_capacity(len as usize);
        file.read_to_end(&mut buf)?;
        Ok(MediaResponse {
            status: 200,
            mime,
            content_length: buf.len() as u64,
            content_range: None,
            body: buf,
        })
    })
    .await
    .map_err(|e| MediaError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
}

pub async fn handle_protocol_request(
    db: &Db,
    request: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    let head_only = request.method() == tauri::http::Method::HEAD;

    if request.method() != tauri::http::Method::GET && !head_only {
        return error_response(405, "Method Not Allowed", head_only);
    }

    let asset_id = match parse_asset_id(request.uri().path()) {
        Some(id) => id,
        None => return error_response(404, "Not Found", head_only),
    };

    let range = request
        .headers()
        .get(tauri::http::header::RANGE)
        .and_then(|v| v.to_str().ok());

    match build_media_response(db, asset_id, range, head_only).await {
        Ok(resp) => build_http_response(resp),
        Err(MediaError::NotFound) => error_response(404, "Not Found", head_only),
        Err(MediaError::UnsupportedType) => {
            error_response(415, "Unsupported Media Type", head_only)
        }
        Err(MediaError::RangeNotSatisfiable(len)) => {
            let mut builder = tauri::http::Response::builder()
                .status(416)
                .header(tauri::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .header(tauri::http::header::CONTENT_LENGTH, 0);
            if let Some(len) = len {
                builder = builder.header(tauri::http::header::CONTENT_RANGE, format!("bytes */{len}"));
            }
            builder.body(Vec::new()).unwrap()
        }
        Err(MediaError::PayloadTooLarge) => error_response(413, "Payload Too Large", head_only),
        Err(e) => error_response(500, &e.to_string(), head_only),
    }
}

fn parse_asset_id(path: &str) -> Option<i64> {
    let trimmed = path.strip_prefix('/')?;
    if trimmed.is_empty() || trimmed.contains('/') {
        return None;
    }
    if !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let id = trimmed.parse::<i64>().ok()?;
    if id <= 0 {
        return None;
    }
    Some(id)
}

fn build_http_response(resp: MediaResponse) -> tauri::http::Response<Vec<u8>> {
    let mut builder = tauri::http::Response::builder()
        .status(resp.status)
        .header(tauri::http::header::CONTENT_TYPE, resp.mime)
        .header(tauri::http::header::CONTENT_LENGTH, resp.content_length)
        .header(tauri::http::header::ACCEPT_RANGES, "bytes");

    if let Some(cr) = resp.content_range {
        builder = builder.header(tauri::http::header::CONTENT_RANGE, cr);
    }

    builder.body(resp.body).unwrap()
}

fn error_response(status: u16, body: &str, head_only: bool) -> tauri::http::Response<Vec<u8>> {
    let body = if head_only { "" } else { body };
    tauri::http::Response::builder()
        .status(status)
        .header(tauri::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(tauri::http::header::CONTENT_LENGTH, body.len())
        .body(body.as_bytes().to_vec())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_mime_allows_only_selected_audio_and_video_extensions() {
        for (asset_type, ext) in [
            ("audio", "mp3"),
            ("audio", "wav"),
            ("audio", "ogg"),
            ("video", "mp4"),
            ("video", "webm"),
        ] {
            assert!(
                media_mime(asset_type, ext).is_some(),
                "expected {asset_type}/{ext} to be supported"
            );
        }
        assert!(media_mime("audio", "flac").is_none());
        assert!(media_mime("video", "mkv").is_none());
        assert!(media_mime("image", "mp4").is_none());

        // Case-insensitive matching.
        assert_eq!(media_mime("audio", "MP3"), Some("audio/mpeg"));
        assert_eq!(media_mime("video", "WEBM"), Some("video/webm"));
    }

    #[test]
    fn media_range_parses_single_http_byte_range() {
        assert_eq!(
            parse_byte_range(Some("bytes=100-199"), 1000).unwrap(),
            Some((100, 199))
        );
        assert_eq!(
            parse_byte_range(Some("bytes=900-"), 1000).unwrap(),
            Some((900, 999))
        );
        assert!(parse_byte_range(Some("bytes=1000-1001"), 1000).is_err());
        assert!(matches!(
            parse_byte_range(Some("bytes=1000-1001"), 1000).unwrap_err(),
            MediaError::RangeNotSatisfiable(Some(1000))
        ));
        assert!(parse_byte_range(Some("bytes=0-1,4-5"), 1000).is_err());
        assert!(matches!(
            parse_byte_range(Some("bytes=0-1,4-5"), 1000).unwrap_err(),
            MediaError::RangeNotSatisfiable(Some(1000))
        ));
        assert!(parse_byte_range(Some("bytes=100-50"), 1000).is_err());
        assert!(parse_byte_range(Some("bytes=-"), 1000).is_err());
        assert!(parse_byte_range(Some("bytes=-500"), 1000).is_err());
    }
}
