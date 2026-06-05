use anyhow::Context;
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
use std::path::{Path, PathBuf};

pub type Db = SqlitePool;

pub fn resolve_database_path(app_data_dir: &Path) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(app_data_dir)
        .with_context(|| format!("failed to create app data directory: {}", app_data_dir.display()))?;
    Ok(app_data_dir.join("data.sqlite"))
}

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
    let normalized = crate::indexer::normalize_path(std::path::Path::new(path))?;
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO library_folders (name, path, created_at, is_enabled) VALUES (?1, ?2, ?3, 1)"
    )
    .bind(name)
    .bind(&normalized)
    .bind(&now)
    .execute(db)
    .await?;

    Ok(LibraryFolder {
        id: result.last_insert_rowid(),
        name: name.to_string(),
        path: normalized,
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
         WHERE file_name != '.DS_Store'
           AND file_name NOT LIKE '._%'
           AND absolute_path NOT LIKE '%/__MACOSX/%'
           AND absolute_path NOT LIKE '%\\__MACOSX\\%'
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

    sqlx::query("UPDATE tags SET last_used_at = ?1 WHERE name = ?2")
        .bind(&now)
        .bind(&normalized)
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

pub async fn get_folder_by_path(db: &Db, path: &str) -> anyhow::Result<Option<LibraryFolder>> {
    let normalized = crate::indexer::normalize_path(std::path::Path::new(path))?;
    let row = sqlx::query_as::<_, (i64, String, String, String, Option<String>, i64)>(
        "SELECT id, name, path, created_at, last_scanned_at, is_enabled FROM library_folders WHERE path = ?1"
    )
    .bind(&normalized)
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

pub async fn delete_library_folder(db: &Db, id: i64) -> anyhow::Result<bool> {
    // Fail any running scan job first so the worker won't crash on a missing folder
    if let Some(job) = running_scan_job_for_folder(db, id).await? {
        fail_scan_job(db, job.id, "folder deleted").await?;
    }
    let rows = sqlx::query("DELETE FROM library_folders WHERE id = ?1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(rows.rows_affected() > 0)
}

pub async fn count_assets_by_folder(db: &Db, folder_id: i64, path: &str) -> anyhow::Result<(i64, i64, bool)> {
    let row: (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(CASE WHEN is_missing = 1 THEN 1 ELSE 0 END), 0)
         FROM assets
         WHERE library_folder_id = ?1
           AND file_name != '.DS_Store'
           AND file_name NOT LIKE '._%' ESCAPE '\\'
           AND absolute_path NOT LIKE '%/__MACOSX/%' ESCAPE '\\'
           AND absolute_path NOT LIKE '%\\__MACOSX\\%' ESCAPE '\\'"
    )
    .bind(folder_id)
    .fetch_one(db)
    .await?;
    let is_accessible = std::path::Path::new(path).is_dir();
    Ok((row.0, row.1, is_accessible))
}

pub async fn update_folder_last_scanned(db: &Db, id: i64) -> anyhow::Result<()> {
    sqlx::query("UPDATE library_folders SET last_scanned_at = ?1 WHERE id = ?2")
        .bind(Utc::now().to_rfc3339())
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn list_asset_tags_map(db: &Db) -> anyhow::Result<Vec<(i64, String)>> {
    let rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT a.asset_id, t.name FROM asset_tags a JOIN tags t ON a.tag_id = t.id"
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn list_tags(db: &Db) -> anyhow::Result<Vec<crate::models::Tag>> {
    let rows = sqlx::query_as::<_, crate::models::Tag>(
        "SELECT id, name, color FROM tags ORDER BY name"
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn list_recent_tags(db: &Db, limit: i64) -> anyhow::Result<Vec<crate::models::Tag>> {
    let limit = limit.clamp(0, 50);
    sqlx::query_as::<_, crate::models::Tag>(
        "SELECT id, name, color
         FROM tags
         ORDER BY last_used_at DESC, name ASC
         LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn list_asset_tags(db: &Db, asset_id: i64) -> anyhow::Result<Vec<String>> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT t.name FROM asset_tags at JOIN tags t ON at.tag_id = t.id WHERE at.asset_id = ?1 ORDER BY t.name"
    )
    .bind(asset_id)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn list_common_tags(db: &Db, asset_ids: &[i64]) -> anyhow::Result<Vec<String>> {
    if asset_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = asset_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT t.name FROM tags t
         JOIN asset_tags at ON t.id = at.tag_id
         WHERE at.asset_id IN ({})
         GROUP BY t.id, t.name
         HAVING COUNT(DISTINCT at.asset_id) = ?",
        placeholders
    );
    let mut query = sqlx::query_as::<_, (String,)>(&sql);
    for id in asset_ids {
        query = query.bind(id);
    }
    query = query.bind(asset_ids.len() as i64);
    let rows = query.fetch_all(db).await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn list_collections(db: &Db) -> anyhow::Result<Vec<crate::models::Collection>> {
    let rows = sqlx::query_as::<_, crate::models::Collection>(
        "SELECT id, name, description FROM collections ORDER BY name"
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn create_collection(db: &Db, name: &str, description: &str) -> anyhow::Result<crate::models::Collection> {
    let name = name.trim();
    anyhow::ensure!(!name.is_empty(), "collection name cannot be empty");
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO collections (name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)"
    )
    .bind(name)
    .bind(description)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(crate::models::Collection {
        id: result.last_insert_rowid(),
        name: name.to_string(),
        description: description.to_string(),
    })
}

pub async fn add_assets_to_collection(db: &Db, collection_id: i64, asset_ids: &[i64]) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    let mut tx = db.begin().await?;
    for asset_id in asset_ids {
        sqlx::query("INSERT OR IGNORE INTO collection_assets (collection_id, asset_id) VALUES (?1, ?2)")
            .bind(collection_id)
            .bind(asset_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE collections SET updated_at = ?1 WHERE id = ?2")
        .bind(&now)
        .bind(collection_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn remove_asset_from_collection(db: &Db, collection_id: i64, asset_id: i64) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM collection_assets WHERE collection_id = ?1 AND asset_id = ?2")
        .bind(collection_id)
        .bind(asset_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn list_collection_assets(db: &Db, collection_id: i64) -> anyhow::Result<Vec<i64>> {
    let rows = sqlx::query_as::<_, (i64,)>(
        "SELECT asset_id FROM collection_assets WHERE collection_id = ?1"
    )
    .bind(collection_id)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
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

#[cfg(test)]
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
         current_path = ?6
         WHERE id = ?7 AND status = 'running'"
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
        i64, i64, i64, i64, i64, i64, i64, i64, i64, String,
        Option<String>, Option<String>, String, String
    )>(
        "SELECT id, include_images, include_audio, include_video, include_fonts,
                include_models, include_spine, include_psd, generate_psd_thumbnails,
                ignored_directory_names, thumbnail_cache_dir, database_path,
                ignored_extensions, updated_at
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
        thumbnail_cache_dir: row.10,
        database_path: row.11,
        ignored_extensions: row.12,
        updated_at: row.13,
    })
}

pub async fn ensure_app_paths(db: &Db, db_path: &str, thumb_dir: &str) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE scan_settings SET database_path = COALESCE(database_path, ?1),
         thumbnail_cache_dir = COALESCE(thumbnail_cache_dir, ?2) WHERE id = 1"
    )
    .bind(db_path)
    .bind(thumb_dir)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn save_scan_settings(db: &Db, settings: &ScanSettings) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE scan_settings SET
         include_images = ?1, include_audio = ?2, include_video = ?3, include_fonts = ?4,
         include_models = ?5, include_spine = ?6, include_psd = ?7,
         generate_psd_thumbnails = ?8, ignored_directory_names = ?9,
         thumbnail_cache_dir = ?10, database_path = ?11,
         ignored_extensions = ?12, updated_at = ?13
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
    .bind(&settings.thumbnail_cache_dir)
    .bind(&settings.database_path)
    .bind(&settings.ignored_extensions)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn update_asset_note(db: &Db, asset_id: i64, note: &str) -> anyhow::Result<Asset> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE assets SET note = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(note)
        .bind(&now)
        .bind(asset_id)
        .execute(db)
        .await?;

    let row = sqlx::query_as::<_, Asset>(
        "SELECT id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size,
                modified_at, width, height, thumbnail_path, thumbnail_status, thumbnail_error, note, is_favorite, is_missing,
                created_at, updated_at
         FROM assets WHERE id = ?1"
    )
    .bind(asset_id)
    .fetch_one(db)
    .await?;

    Ok(row)
}

pub async fn record_recent_asset_action(
    db: &Db,
    asset_id: i64,
    action_type: &str,
) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO recent_asset_actions (asset_id, action_type, created_at) VALUES (?1, ?2, ?3)",
    )
    .bind(asset_id)
    .bind(action_type)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn list_recent_asset_actions(
    db: &Db,
    limit: i64,
) -> anyhow::Result<Vec<crate::models::RecentAssetAction>> {
    sqlx::query_as::<_, crate::models::RecentAssetAction>(
        "SELECT id, asset_id, action_type, created_at
         FROM recent_asset_actions
         ORDER BY created_at DESC
         LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ensure_app_paths_sets_defaults_once() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE scan_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            include_images INTEGER NOT NULL DEFAULT 1,
            include_audio INTEGER NOT NULL DEFAULT 1,
            include_video INTEGER NOT NULL DEFAULT 1,
            include_fonts INTEGER NOT NULL DEFAULT 1,
            include_models INTEGER NOT NULL DEFAULT 1,
            include_spine INTEGER NOT NULL DEFAULT 1,
            include_psd INTEGER NOT NULL DEFAULT 1,
            generate_psd_thumbnails INTEGER NOT NULL DEFAULT 0,
            ignored_directory_names TEXT NOT NULL DEFAULT '',
            thumbnail_cache_dir TEXT,
            database_path TEXT,
            ignored_extensions TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL
        )").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO scan_settings (id, updated_at) VALUES (1, 'now')")
            .execute(&pool).await.unwrap();

        ensure_app_paths(&pool, "first.db", "first-thumbs").await.unwrap();
        ensure_app_paths(&pool, "second.db", "second-thumbs").await.unwrap();

        let row: (String, String) = sqlx::query_as(
            "SELECT database_path, thumbnail_cache_dir FROM scan_settings WHERE id = 1"
        ).fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, "first.db");
        assert_eq!(row.1, "first-thumbs");
    }
}

#[cfg(test)]
mod collections_tests {
    use super::*;

    async fn setup_db() -> Db {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE collections (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, description TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL, updated_at TEXT NOT NULL)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE collection_assets (collection_id INTEGER NOT NULL, asset_id INTEGER NOT NULL, PRIMARY KEY (collection_id, asset_id))")
            .execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn create_collection_valid_name() {
        let db = setup_db().await;
        let col = create_collection(&db, "My Collection", "desc").await.unwrap();
        assert_eq!(col.name, "My Collection");
        assert_eq!(col.description, "desc");
        assert!(col.id > 0);
    }

    #[tokio::test]
    async fn create_collection_trims_whitespace() {
        let db = setup_db().await;
        let col = create_collection(&db, "  Trimmed  ", "").await.unwrap();
        assert_eq!(col.name, "Trimmed");
    }

    #[tokio::test]
    async fn create_collection_empty_name_fails() {
        let db = setup_db().await;
        let err = create_collection(&db, "", "").await.unwrap_err();
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[tokio::test]
    async fn create_collection_whitespace_only_fails() {
        let db = setup_db().await;
        let err = create_collection(&db, "   ", "").await.unwrap_err();
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[tokio::test]
    async fn list_collections_returns_sorted() {
        let db = setup_db().await;
        create_collection(&db, "B", "").await.unwrap();
        create_collection(&db, "A", "").await.unwrap();
        create_collection(&db, "C", "").await.unwrap();
        let cols = list_collections(&db).await.unwrap();
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].name, "A");
        assert_eq!(cols[1].name, "B");
        assert_eq!(cols[2].name, "C");
    }

    #[tokio::test]
    async fn add_and_list_collection_assets() {
        let db = setup_db().await;
        let col = create_collection(&db, "Test", "").await.unwrap();
        add_assets_to_collection(&db, col.id, &[1, 2, 3]).await.unwrap();
        let ids = list_collection_assets(&db, col.id).await.unwrap();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
    }

    #[tokio::test]
    async fn add_duplicate_assets_is_idempotent() {
        let db = setup_db().await;
        let col = create_collection(&db, "Test", "").await.unwrap();
        add_assets_to_collection(&db, col.id, &[1, 2]).await.unwrap();
        add_assets_to_collection(&db, col.id, &[1, 3]).await.unwrap();
        let ids = list_collection_assets(&db, col.id).await.unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[tokio::test]
    async fn remove_asset_from_collection_works() {
        let db = setup_db().await;
        let col = create_collection(&db, "Test", "").await.unwrap();
        add_assets_to_collection(&db, col.id, &[1, 2]).await.unwrap();
        remove_asset_from_collection(&db, col.id, 1).await.unwrap();
        let ids = list_collection_assets(&db, col.id).await.unwrap();
        assert_eq!(ids, vec![2]);
    }
}

#[cfg(test)]
mod scan_job_progress_tests {
    use super::*;

    async fn setup_db() -> Db {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE scan_jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_folder_id INTEGER NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                cancelled_at TEXT,
                found_count INTEGER NOT NULL DEFAULT 0,
                added_count INTEGER NOT NULL DEFAULT 0,
                updated_count INTEGER NOT NULL DEFAULT 0,
                unchanged_count INTEGER NOT NULL DEFAULT 0,
                missing_count INTEGER NOT NULL DEFAULT 0,
                skipped_count INTEGER NOT NULL DEFAULT 0,
                current_path TEXT,
                error_message TEXT
            )"
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    async fn insert_job(db: &Db, status: &str) -> i64 {
        sqlx::query("INSERT INTO scan_jobs (library_folder_id, status, started_at, found_count, added_count, updated_count, unchanged_count, skipped_count, current_path) VALUES (1, ?1, 'now', 1, 2, 3, 4, 5, 'old')")
            .bind(status)
            .execute(db)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn progress_row(db: &Db, job_id: i64) -> (i64, i64, i64, i64, i64, Option<String>) {
        sqlx::query_as(
            "SELECT found_count, added_count, updated_count, unchanged_count, skipped_count, current_path
             FROM scan_jobs WHERE id = ?1"
        )
        .bind(job_id)
        .fetch_one(db)
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn update_scan_job_progress_updates_running_job() {
        let db = setup_db().await;
        let job_id = insert_job(&db, "running").await;

        update_scan_job_progress(&db, job_id, 10, 11, 12, 13, 14, Some("new"))
            .await
            .unwrap();

        assert_eq!(
            progress_row(&db, job_id).await,
            (10, 11, 12, 13, 14, Some("new".to_string()))
        );
    }

    #[tokio::test]
    async fn update_scan_job_progress_does_not_modify_cancelled_job() {
        let db = setup_db().await;
        let job_id = insert_job(&db, "cancelled").await;

        update_scan_job_progress(&db, job_id, 10, 11, 12, 13, 14, Some("new"))
            .await
            .unwrap();

        assert_eq!(
            progress_row(&db, job_id).await,
            (1, 2, 3, 4, 5, Some("old".to_string()))
        );
    }

    #[tokio::test]
    async fn update_scan_job_progress_does_not_modify_failed_or_completed_jobs() {
        let db = setup_db().await;
        let failed_id = insert_job(&db, "failed").await;
        let completed_id = insert_job(&db, "completed").await;

        update_scan_job_progress(&db, failed_id, 10, 11, 12, 13, 14, Some("new"))
            .await
            .unwrap();
        update_scan_job_progress(&db, completed_id, 20, 21, 22, 23, 24, Some("newer"))
            .await
            .unwrap();

        assert_eq!(
            progress_row(&db, failed_id).await,
            (1, 2, 3, 4, 5, Some("old".to_string()))
        );
        assert_eq!(
            progress_row(&db, completed_id).await,
            (1, 2, 3, 4, 5, Some("old".to_string()))
        );
    }
}

#[cfg(test)]
mod list_assets_filter_tests {
    use super::*;

    async fn setup_db() -> Db {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE assets (
                id INTEGER PRIMARY KEY,
                library_folder_id INTEGER NOT NULL,
                absolute_path TEXT NOT NULL UNIQUE,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                file_size INTEGER NOT NULL DEFAULT 0,
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
                updated_at TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn list_assets_excludes_macos_metadata_files() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, created_at, updated_at) VALUES
            (1, 1, '/test/hero.png', 'hero.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (2, 1, '/test/.DS_Store', '.DS_Store', '', 'other', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (3, 1, '/test/__MACOSX/._hero.png', '._hero.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (4, 1, '/test/._something.jpg', '._something.jpg', 'jpg', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
            .execute(&db).await.unwrap();

        let assets = list_assets(&db, 100, 0).await.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].file_name, "hero.png");
    }

    #[tokio::test]
    async fn list_assets_excludes_macosx_on_windows_paths() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, created_at, updated_at) VALUES
            (1, 1, 'G:/assets/__MACOSX/._icon.png', '._icon.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (2, 1, 'G:/assets/icon.png', 'icon.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
            .execute(&db).await.unwrap();

        let assets = list_assets(&db, 100, 0).await.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].file_name, "icon.png");
    }
}

#[cfg(test)]
mod folder_management_tests {
    use super::*;

    async fn setup_db() -> Db {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE library_folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                last_scanned_at TEXT,
                is_enabled INTEGER NOT NULL DEFAULT 1
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE assets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_folder_id INTEGER NOT NULL,
                absolute_path TEXT NOT NULL UNIQUE,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                file_size INTEGER NOT NULL DEFAULT 0,
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
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#5B8DEF',
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE asset_tags (
                asset_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (asset_id, tag_id),
                FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE collections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE collection_assets (
                collection_id INTEGER NOT NULL,
                asset_id INTEGER NOT NULL,
                PRIMARY KEY (collection_id, asset_id),
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
                FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE scan_jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_folder_id INTEGER NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                cancelled_at TEXT,
                found_count INTEGER NOT NULL DEFAULT 0,
                added_count INTEGER NOT NULL DEFAULT 0,
                updated_count INTEGER NOT NULL DEFAULT 0,
                unchanged_count INTEGER NOT NULL DEFAULT 0,
                missing_count INTEGER NOT NULL DEFAULT 0,
                skipped_count INTEGER NOT NULL DEFAULT 0,
                current_path TEXT,
                error_message TEXT,
                FOREIGN KEY (library_folder_id) REFERENCES library_folders(id) ON DELETE CASCADE
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn count_assets_by_folder_returns_correct_totals() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO library_folders (id, name, path, created_at, is_enabled) VALUES (1, 'Test', '/test', '2024-01-01T00:00:00Z', 1)")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, created_at, updated_at, is_missing) VALUES
            (1, 1, '/test/a.png', 'a.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', 0),
            (2, 1, '/test/b.png', 'b.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', 1),
            (3, 1, '/test/c.png', 'c.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', 1)")
            .execute(&db).await.unwrap();

        let (total, missing, is_accessible) = count_assets_by_folder(&db, 1, "/nonexistent_path").await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(missing, 2);
        assert_eq!(is_accessible, false);

        let (total, missing, is_accessible) = count_assets_by_folder(&db, 1, ".").await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(missing, 2);
        assert_eq!(is_accessible, true);
    }

    #[tokio::test]
    async fn count_assets_by_folder_excludes_macos_metadata() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO library_folders (id, name, path, created_at, is_enabled) VALUES (1, 'Test', '/test', '2024-01-01T00:00:00Z', 1)")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, created_at, updated_at, is_missing) VALUES
            (1, 1, '/test/hero.png', 'hero.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', 0),
            (2, 1, '/test/.DS_Store', '.DS_Store', '', 'other', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', 0),
            (3, 1, '/test/._hero.png', '._hero.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', 0),
            (4, 1, '/test/__MACOSX/._icon.png', '._icon.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', 0)")
            .execute(&db).await.unwrap();

        let (total, missing, _) = count_assets_by_folder(&db, 1, "/test").await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(missing, 0);
    }

    #[tokio::test]
    async fn delete_library_folder_cascades_to_related_records() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO library_folders (id, name, path, created_at, is_enabled) VALUES (1, 'Test', '/test', '2024-01-01T00:00:00Z', 1)")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, created_at, updated_at) VALUES
            (1, 1, '/test/a.png', 'a.png', 'png', 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO tags (id, name, color, created_at, last_used_at) VALUES (1, 'tag1', '#000000', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO asset_tags (asset_id, tag_id) VALUES (1, 1)")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO collections (id, name, description, created_at, updated_at) VALUES (1, 'Col1', '', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO collection_assets (collection_id, asset_id) VALUES (1, 1)")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO scan_jobs (id, library_folder_id, status, started_at) VALUES (1, 1, 'completed', '2024-01-01T00:00:00Z')")
            .execute(&db).await.unwrap();

        let deleted = delete_library_folder(&db, 1).await.unwrap();
        assert!(deleted);

        let folder_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM library_folders WHERE id = 1")
            .fetch_one(&db).await.unwrap();
        assert_eq!(folder_count.0, 0);

        let asset_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM assets WHERE library_folder_id = 1")
            .fetch_one(&db).await.unwrap();
        assert_eq!(asset_count.0, 0);

        let tag_link_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset_tags WHERE asset_id = 1")
            .fetch_one(&db).await.unwrap();
        assert_eq!(tag_link_count.0, 0);

        let col_link_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM collection_assets WHERE asset_id = 1")
            .fetch_one(&db).await.unwrap();
        assert_eq!(col_link_count.0, 0);

        let scan_job_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scan_jobs WHERE library_folder_id = 1")
            .fetch_one(&db).await.unwrap();
        assert_eq!(scan_job_count.0, 0);
    }
}

#[cfg(test)]
mod recent_action_tests {
    use super::*;

    async fn setup_db() -> Db {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE library_folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                last_scanned_at TEXT,
                is_enabled INTEGER NOT NULL DEFAULT 1
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE assets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_folder_id INTEGER NOT NULL,
                absolute_path TEXT NOT NULL UNIQUE,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                file_size INTEGER NOT NULL DEFAULT 0,
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
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE recent_asset_actions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                asset_id INTEGER NOT NULL,
                action_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn records_recent_asset_action() {
        let db = setup_db().await;
        let folder = create_library_folder(&db, "fixture", "C:/assets").await.unwrap();
        sqlx::query("INSERT INTO assets (library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
            .bind(folder.id)
            .bind("C:/assets/icon.png")
            .bind("icon.png")
            .bind("png")
            .bind("image")
            .bind("2024-01-01T00:00:00Z")
            .bind("2024-01-01T00:00:00Z")
            .bind("2024-01-01T00:00:00Z")
            .execute(&db).await.unwrap();
        let asset_id = sqlx::query_as::<_, (i64,)>("SELECT id FROM assets WHERE absolute_path = ?1")
            .bind("C:/assets/icon.png")
            .fetch_one(&db).await.unwrap().0;

        record_recent_asset_action(&db, asset_id, "copy_path").await.unwrap();
        let actions = list_recent_asset_actions(&db, 10).await.unwrap();

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].asset_id, asset_id);
        assert_eq!(actions[0].action_type, "copy_path");
    }
}

#[cfg(test)]
mod recent_tag_tests {
    use super::*;

    async fn setup_db() -> Db {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#5B8DEF',
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn lists_recent_tags_by_last_used_then_name() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('角色', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-03T00:00:00Z')")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('地形', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-04T00:00:00Z')")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('特效', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-03T00:00:00Z')")
            .execute(&db).await.unwrap();

        let tags = list_recent_tags(&db, 3).await.unwrap();

        let names: Vec<String> = tags.into_iter().map(|tag| tag.name).collect();
        assert_eq!(names, vec!["地形", "特效", "角色"]);
    }

    #[tokio::test]
    async fn clamps_recent_tag_limit() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('地形', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-04T00:00:00Z')")
            .execute(&db).await.unwrap();

        let tags = list_recent_tags(&db, -5).await.unwrap();

        assert!(tags.is_empty());
    }

    #[tokio::test]
    async fn reusing_existing_tag_updates_last_used_at() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('旧标签', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('新标签', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-04T00:00:00Z')")
            .execute(&db).await.unwrap();

        // 复用旧标签，触发 last_used_at 刷新
        create_or_get_tag(&db, "旧标签").await.unwrap();

        let tags = list_recent_tags(&db, 10).await.unwrap();
        let names: Vec<String> = tags.into_iter().map(|tag| tag.name).collect();
        assert_eq!(names, vec!["旧标签", "新标签"]);
    }
}
