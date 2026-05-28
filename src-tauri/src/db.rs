use anyhow::Context;
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
use std::path::Path;

pub type Db = SqlitePool;

pub async fn connect(database_path: &Path) -> anyhow::Result<Db> {
    if let Some(parent) = database_path.parent() {
        std::fs::create_dir_all(parent).context("failed to create database directory")?;
    }

    let options = SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .context("failed to connect sqlite database")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    Ok(pool)
}

use chrono::Utc;
use crate::models::{Asset, LibraryFolder};

pub async fn create_library_folder(db: &Db, name: &str, path: &str) -> anyhow::Result<LibraryFolder> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO library_folders (name, path, created_at, is_enabled) VALUES (?1, ?2, ?3, 1)"
    )
    .bind(name)
    .bind(path)
    .bind(&now)
    .execute(db)
    .await?;

    Ok(LibraryFolder {
        id: result.last_insert_rowid(),
        name: name.to_string(),
        path: path.to_string(),
        created_at: now,
        last_scanned_at: None,
        is_enabled: true,
    })
}

pub async fn list_library_folders(db: &Db) -> anyhow::Result<Vec<LibraryFolder>> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, Option<String>, i64)>(
        "SELECT id, name, path, created_at, last_scanned_at, is_enabled FROM library_folders ORDER BY name"
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| LibraryFolder {
            id: row.0,
            name: row.1,
            path: row.2,
            created_at: row.3,
            last_scanned_at: row.4,
            is_enabled: row.5 == 1,
        })
        .collect())
}

pub async fn list_assets(db: &Db, limit: i64, offset: i64) -> anyhow::Result<Vec<Asset>> {
    let rows = sqlx::query_as::<_, Asset>(
        "SELECT id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size,
                modified_at, width, height, thumbnail_path, thumbnail_status, thumbnail_error, note, is_favorite, is_missing,
                created_at, updated_at
         FROM assets
         ORDER BY file_name
         LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    Ok(rows)
}

use crate::models::{ScanJob, ScanJobStatus, ScanSettings};
use crate::indexer::ScannedAsset;

pub async fn upsert_scanned_asset(
    db: &Db,
    library_folder_id: i64,
    asset: &ScannedAsset,
    thumbnail_path: Option<&str>,
    width: Option<i64>,
    height: Option<i64>,
) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO assets (
            library_folder_id, absolute_path, file_name, extension, asset_type, file_size,
            modified_at, width, height, thumbnail_path, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(absolute_path) DO UPDATE SET
            file_name = excluded.file_name,
            extension = excluded.extension,
            asset_type = excluded.asset_type,
            file_size = excluded.file_size,
            modified_at = excluded.modified_at,
            width = excluded.width,
            height = excluded.height,
            thumbnail_path = excluded.thumbnail_path,
            is_missing = 0,
            updated_at = excluded.updated_at"
    )
    .bind(library_folder_id)
    .bind(&asset.absolute_path)
    .bind(&asset.file_name)
    .bind(&asset.extension)
    .bind(&asset.asset_type)
    .bind(asset.file_size)
    .bind(&asset.modified_at)
    .bind(width)
    .bind(height)
    .bind(thumbnail_path)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    Ok(())
}

use crate::tags::normalize_tag_name;

pub async fn set_asset_favorite(db: &Db, asset_id: i64, is_favorite: bool) -> anyhow::Result<()> {
    sqlx::query("UPDATE assets SET is_favorite = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(if is_favorite { 1 } else { 0 })
        .bind(Utc::now().to_rfc3339())
        .bind(asset_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn create_or_get_tag(db: &Db, name: &str) -> anyhow::Result<i64> {
    let normalized = normalize_tag_name(name);
    anyhow::ensure!(!normalized.is_empty(), "tag name cannot be empty");
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT OR IGNORE INTO tags (name, color, created_at, last_used_at) VALUES (?1, '#5B8DEF', ?2, ?3)"
    )
    .bind(&normalized)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    let id: (i64,) = sqlx::query_as("SELECT id FROM tags WHERE name = ?1")
        .bind(&normalized)
        .fetch_one(db)
        .await?;
    Ok(id.0)
}

pub async fn apply_tag_to_assets(db: &Db, tag_name: &str, asset_ids: &[i64]) -> anyhow::Result<()> {
    let tag_id = create_or_get_tag(db, tag_name).await?;
    for asset_id in asset_ids {
        sqlx::query("INSERT OR IGNORE INTO asset_tags (asset_id, tag_id) VALUES (?1, ?2)")
            .bind(asset_id)
            .bind(tag_id)
            .execute(db)
            .await?;
    }
    Ok(())
}

pub async fn get_folder_by_id(db: &Db, id: i64) -> anyhow::Result<Option<LibraryFolder>> {
    let row = sqlx::query_as::<_, (i64, String, String, String, Option<String>, i64)>(
        "SELECT id, name, path, created_at, last_scanned_at, is_enabled FROM library_folders WHERE id = ?1"
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|row| LibraryFolder {
        id: row.0,
        name: row.1,
        path: row.2,
        created_at: row.3,
        last_scanned_at: row.4,
        is_enabled: row.5 == 1,
    }))
}

pub async fn update_folder_last_scanned(db: &Db, id: i64) -> anyhow::Result<()> {
    sqlx::query("UPDATE library_folders SET last_scanned_at = ?1 WHERE id = ?2")
        .bind(Utc::now().to_rfc3339())
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn mark_missing_assets(db: &Db, folder_id: i64, existing_paths: &[String]) -> anyhow::Result<u64> {
    let placeholders: Vec<String> = existing_paths.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "UPDATE assets SET is_missing = 1, updated_at = ?1 WHERE library_folder_id = ?2 AND absolute_path NOT IN ({})",
        placeholders.join(",")
    );
    let mut query = sqlx::query(&sql)
        .bind(Utc::now().to_rfc3339())
        .bind(folder_id);
    for path in existing_paths {
        query = query.bind(path);
    }
    let result = query.execute(db).await?;
    Ok(result.rows_affected())
}

pub async fn list_asset_tags_map(db: &Db) -> anyhow::Result<Vec<(i64, String)>> {
    let rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT a.asset_id, t.name FROM asset_tags a JOIN tags t ON a.tag_id = t.id"
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_scan_job(db: &Db, folder_id: i64) -> anyhow::Result<ScanJob> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO scan_jobs (library_folder_id, status, started_at) VALUES (?1, 'running', ?2)"
    )
    .bind(folder_id)
    .bind(&now)
    .execute(db)
    .await?;

    Ok(ScanJob {
        id: result.last_insert_rowid(),
        library_folder_id: folder_id,
        status: ScanJobStatus::Running,
        started_at: now,
        finished_at: None,
        cancelled_at: None,
        found_count: 0,
        added_count: 0,
        updated_count: 0,
        unchanged_count: 0,
        missing_count: 0,
        skipped_count: 0,
        current_path: None,
        error_message: None,
    })
}

pub async fn get_scan_job(db: &Db, job_id: i64) -> anyhow::Result<Option<ScanJob>> {
    let row = sqlx::query_as::<_, (
        i64, i64, String, String, Option<String>, Option<String>, i64,
        i64, i64, i64, i64, i64, Option<String>, Option<String>
    )>(
        "SELECT id, library_folder_id, status, started_at, finished_at, cancelled_at,
                found_count, added_count, updated_count, unchanged_count, missing_count,
                skipped_count, current_path, error_message
         FROM scan_jobs WHERE id = ?1"
    )
    .bind(job_id)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| ScanJob {
        id: r.0,
        library_folder_id: r.1,
        status: r.2.parse().expect("invalid status"),
        started_at: r.3,
        finished_at: r.4,
        cancelled_at: r.5,
        found_count: r.6,
        added_count: r.7,
        updated_count: r.8,
        unchanged_count: r.9,
        missing_count: r.10,
        skipped_count: r.11,
        current_path: r.12,
        error_message: r.13,
    }))
}

pub async fn latest_scan_job_for_folder(db: &Db, folder_id: i64) -> anyhow::Result<Option<ScanJob>> {
    let row = sqlx::query_as::<_, (
        i64, i64, String, String, Option<String>, Option<String>, i64,
        i64, i64, i64, i64, i64, Option<String>, Option<String>
    )>(
        "SELECT id, library_folder_id, status, started_at, finished_at, cancelled_at,
                found_count, added_count, updated_count, unchanged_count, missing_count,
                skipped_count, current_path, error_message
         FROM scan_jobs WHERE library_folder_id = ?1 ORDER BY started_at DESC LIMIT 1"
    )
    .bind(folder_id)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| ScanJob {
        id: r.0,
        library_folder_id: r.1,
        status: r.2.parse().expect("invalid status"),
        started_at: r.3,
        finished_at: r.4,
        cancelled_at: r.5,
        found_count: r.6,
        added_count: r.7,
        updated_count: r.8,
        unchanged_count: r.9,
        missing_count: r.10,
        skipped_count: r.11,
        current_path: r.12,
        error_message: r.13,
    }))
}

pub async fn running_scan_job_for_folder(db: &Db, folder_id: i64) -> anyhow::Result<Option<ScanJob>> {
    let row = sqlx::query_as::<_, (
        i64, i64, String, String, Option<String>, Option<String>, i64,
        i64, i64, i64, i64, i64, Option<String>, Option<String>
    )>(
        "SELECT id, library_folder_id, status, started_at, finished_at, cancelled_at,
                found_count, added_count, updated_count, unchanged_count, missing_count,
                skipped_count, current_path, error_message
         FROM scan_jobs WHERE library_folder_id = ?1 AND status = 'running' ORDER BY started_at DESC LIMIT 1"
    )
    .bind(folder_id)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| ScanJob {
        id: r.0,
        library_folder_id: r.1,
        status: r.2.parse().expect("invalid status"),
        started_at: r.3,
        finished_at: r.4,
        cancelled_at: r.5,
        found_count: r.6,
        added_count: r.7,
        updated_count: r.8,
        unchanged_count: r.9,
        missing_count: r.10,
        skipped_count: r.11,
        current_path: r.12,
        error_message: r.13,
    }))
}

pub async fn update_scan_job_progress(
    db: &Db,
    job_id: i64,
    found: i64,
    added: i64,
    updated: i64,
    unchanged: i64,
    skipped: i64,
    current_path: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE scan_jobs SET found_count = ?1, added_count = ?2, updated_count = ?3,
         unchanged_count = ?4, skipped_count = ?5,
         current_path = CASE WHEN status = 'running' THEN ?6 ELSE current_path END
         WHERE id = ?7"
    )
    .bind(found)
    .bind(added)
    .bind(updated)
    .bind(unchanged)
    .bind(skipped)
    .bind(current_path)
    .bind(job_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn finish_scan_job(db: &Db, job_id: i64, missing: i64) -> anyhow::Result<u64> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE scan_jobs SET status = 'completed', finished_at = ?1, missing_count = ?2, current_path = NULL WHERE id = ?3 AND status = 'running'"
    )
    .bind(&now)
    .bind(missing)
    .bind(job_id)
    .execute(db)
    .await?;
    if result.rows_affected() > 0 {
        sqlx::query("DELETE FROM scan_seen_paths WHERE scan_job_id = ?1")
            .bind(job_id)
            .execute(db)
            .await?;
    }
    Ok(result.rows_affected())
}

pub async fn fail_scan_job(db: &Db, job_id: i64, message: &str) -> anyhow::Result<u64> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE scan_jobs SET status = 'failed', finished_at = ?1, error_message = ?2, current_path = NULL WHERE id = ?3 AND status = 'running'"
    )
    .bind(&now)
    .bind(message)
    .bind(job_id)
    .execute(db)
    .await?;
    if result.rows_affected() > 0 {
        sqlx::query("DELETE FROM scan_seen_paths WHERE scan_job_id = ?1")
            .bind(job_id)
            .execute(db)
            .await?;
    }
    Ok(result.rows_affected())
}

pub async fn cancel_scan_job(db: &Db, job_id: i64) -> anyhow::Result<u64> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE scan_jobs SET status = 'cancelled', finished_at = ?1, cancelled_at = ?2, current_path = NULL WHERE id = ?3 AND status = 'running'"
    )
    .bind(&now)
    .bind(&now)
    .bind(job_id)
    .execute(db)
    .await?;
    if result.rows_affected() > 0 {
        sqlx::query("DELETE FROM scan_seen_paths WHERE scan_job_id = ?1")
            .bind(job_id)
            .execute(db)
            .await?;
    }
    Ok(result.rows_affected())
}

pub async fn cleanup_old_scan_jobs(db: &Db, retain_per_folder: i64) -> anyhow::Result<u64> {
    let result = sqlx::query(
        "DELETE FROM scan_jobs WHERE id IN (
            SELECT sj.id FROM scan_jobs sj
            WHERE sj.status IN ('completed', 'cancelled', 'failed')
              AND (
                SELECT COUNT(*)
                FROM scan_jobs sj2
                WHERE sj2.library_folder_id = sj.library_folder_id
                  AND sj2.status IN ('completed', 'cancelled', 'failed')
                  AND (sj2.finished_at > sj.finished_at
                       OR (sj2.finished_at = sj.finished_at AND sj2.id > sj.id))
              ) >= ?1
        )"
    )
    .bind(retain_per_folder)
    .execute(db)
    .await?;
    Ok(result.rows_affected())
}

pub async fn add_seen_paths(db: &Db, job_id: i64, paths: &[String]) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;
    for path in paths {
        sqlx::query(
            "INSERT OR IGNORE INTO scan_seen_paths (scan_job_id, absolute_path) VALUES (?1, ?2)"
        )
        .bind(job_id)
        .bind(path)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn mark_missing_assets_from_seen(db: &Db, folder_id: i64, job_id: i64) -> anyhow::Result<u64> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE assets
         SET is_missing = 1, updated_at = ?1
         WHERE library_folder_id = ?2
           AND absolute_path NOT IN (
             SELECT absolute_path FROM scan_seen_paths WHERE scan_job_id = ?3
           )
           AND EXISTS (SELECT 1 FROM scan_jobs WHERE id = ?3 AND status = 'running')"
    )
    .bind(&now)
    .bind(folder_id)
    .bind(job_id)
    .execute(db)
    .await?;
    Ok(result.rows_affected())
}

pub async fn get_scan_settings(db: &Db) -> anyhow::Result<ScanSettings> {
    let row = sqlx::query_as::<_, (
        i64, i64, i64, i64, i64, i64, i64, i64, i64, String, String
    )>(
        "SELECT id, include_images, include_audio, include_video, include_fonts,
                include_models, include_spine, include_psd, generate_psd_thumbnails,
                ignored_directory_names, updated_at
         FROM scan_settings WHERE id = 1"
    )
    .fetch_one(db)
    .await?;

    Ok(ScanSettings {
        id: row.0,
        include_images: row.1 == 1,
        include_audio: row.2 == 1,
        include_video: row.3 == 1,
        include_fonts: row.4 == 1,
        include_models: row.5 == 1,
        include_spine: row.6 == 1,
        include_psd: row.7 == 1,
        generate_psd_thumbnails: row.8 == 1,
        ignored_directory_names: row.9,
        updated_at: row.10,
    })
}

pub async fn save_scan_settings(db: &Db, settings: &ScanSettings) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE scan_settings SET
         include_images = ?1, include_audio = ?2, include_video = ?3, include_fonts = ?4,
         include_models = ?5, include_spine = ?6, include_psd = ?7,
         generate_psd_thumbnails = ?8, ignored_directory_names = ?9, updated_at = ?10
         WHERE id = 1"
    )
    .bind(if settings.include_images { 1 } else { 0 })
    .bind(if settings.include_audio { 1 } else { 0 })
    .bind(if settings.include_video { 1 } else { 0 })
    .bind(if settings.include_fonts { 1 } else { 0 })
    .bind(if settings.include_models { 1 } else { 0 })
    .bind(if settings.include_spine { 1 } else { 0 })
    .bind(if settings.include_psd { 1 } else { 0 })
    .bind(if settings.generate_psd_thumbnails { 1 } else { 0 })
    .bind(&settings.ignored_directory_names)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_thumbnail_failed(db: &Db, asset_id: i64, message: &str) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE assets SET thumbnail_status = 'failed', thumbnail_error = ?1, updated_at = ?2 WHERE id = ?3"
    )
    .bind(message)
    .bind(Utc::now().to_rfc3339())
    .bind(asset_id)
    .execute(db)
    .await?;
    Ok(())
}

