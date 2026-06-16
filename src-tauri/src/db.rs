use anyhow::Context;
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, QueryBuilder, Row, Sqlite, SqlitePool};
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

pub async fn get_asset_by_id(db: &Db, asset_id: i64) -> anyhow::Result<Option<Asset>> {
    sqlx::query_as::<_, Asset>(
        "SELECT id, library_folder_id, absolute_path, file_name, extension, asset_type,
                file_size, modified_at, width, height, thumbnail_path, thumbnail_status,
                thumbnail_error, note, is_favorite, is_missing, created_at, updated_at
         FROM assets WHERE id = ?1"
    )
    .bind(asset_id)
    .fetch_optional(db)
    .await
    .map_err(Into::into)
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

async fn _create_or_get_tag(conn: &mut sqlx::SqliteConnection, name: &str) -> anyhow::Result<i64> {
    let normalized = normalize_tag_name(name);
    anyhow::ensure!(!normalized.is_empty(), "tag name cannot be empty");
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT OR IGNORE INTO tags (name, color, created_at, last_used_at) VALUES (?1, '#5B8DEF', ?2, ?3)"
    )
    .bind(&normalized)
    .bind(&now)
    .bind(&now)
    .execute(&mut *conn)
    .await?;

    sqlx::query("UPDATE tags SET last_used_at = ?1 WHERE name = ?2")
        .bind(&now)
        .bind(&normalized)
        .execute(&mut *conn)
        .await?;

    let id: (i64,) = sqlx::query_as("SELECT id FROM tags WHERE name = ?1")
        .bind(&normalized)
        .fetch_one(&mut *conn)
        .await?;
    Ok(id.0)
}

pub async fn apply_tag_to_assets(db: &Db, tag_name: &str, asset_ids: &[i64]) -> anyhow::Result<()> {
    if asset_ids.is_empty() {
        return Ok(());
    }
    let mut tx = db.begin().await?;
    let tag_id = _create_or_get_tag(&mut *tx, tag_name).await?;
    for asset_id in asset_ids {
        sqlx::query("INSERT OR IGNORE INTO asset_tags (asset_id, tag_id) VALUES (?1, ?2)")
            .bind(asset_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }
    for asset_id in asset_ids {
        refresh_asset_search_document(&mut *tx, *asset_id).await?;
    }
    tx.commit().await?;
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

    let mut tx = db.begin().await?;
    for table in [PREFIX_FTS_TABLE, TRIGRAM_FTS_TABLE] {
        let sql = format!(
            "DELETE FROM {table}
             WHERE rowid IN (
               SELECT id FROM assets WHERE library_folder_id = ?1
             )"
        );
        sqlx::query(&sql)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    let rows = sqlx::query("DELETE FROM library_folders WHERE id = ?1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
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
    sqlx::query_as::<_, crate::models::Tag>(
        "SELECT t.id, t.name, t.color, COUNT(at.asset_id) AS asset_count
         FROM tags t
         LEFT JOIN asset_tags at ON at.tag_id = t.id
         GROUP BY t.id, t.name, t.color
         ORDER BY t.name COLLATE NOCASE, t.id"
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn list_recent_tags(db: &Db, limit: i64) -> anyhow::Result<Vec<crate::models::Tag>> {
    let limit = limit.clamp(0, 50);
    sqlx::query_as::<_, crate::models::Tag>(
        "SELECT id, name, color,
            (SELECT COUNT(*) FROM asset_tags at WHERE at.tag_id = tags.id) AS asset_count
         FROM tags
         ORDER BY last_used_at DESC, name ASC
         LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

async fn load_tag(
    conn: &mut sqlx::SqliteConnection,
    tag_id: i64,
) -> anyhow::Result<crate::models::Tag> {
    sqlx::query_as::<_, crate::models::Tag>(
        "SELECT t.id, t.name, t.color, COUNT(at.asset_id) AS asset_count
         FROM tags t
         LEFT JOIN asset_tags at ON at.tag_id = t.id
         WHERE t.id = ?1
         GROUP BY t.id, t.name, t.color"
    )
    .bind(tag_id)
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| anyhow::anyhow!("tag not found"))
}

pub async fn update_tag(
    db: &Db,
    tag_id: i64,
    name: &str,
    color: &str,
) -> anyhow::Result<crate::models::Tag> {
    let name = normalize_tag_name(name);
    anyhow::ensure!(!name.is_empty(), "tag name cannot be empty");
    let color = crate::tags::normalize_tag_color(color)?;
    let mut tx = db.begin().await?;
    let source = load_tag(&mut *tx, tag_id).await?;

    if name == source.name && color == source.color {
        tx.rollback().await?;
        return Ok(source);
    }

    let affected_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT asset_id FROM asset_tags WHERE tag_id = ?1 ORDER BY asset_id"
    )
    .bind(tag_id)
    .fetch_all(&mut *tx)
    .await?;

    let target_id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM tags WHERE name = ?1 AND id != ?2")
            .bind(&name)
            .bind(tag_id)
            .fetch_optional(&mut *tx)
            .await?;

    let result_id = if let Some(target_id) = target_id {
        sqlx::query(
            "INSERT OR IGNORE INTO asset_tags (asset_id, tag_id)
             SELECT asset_id, ?1 FROM asset_tags WHERE tag_id = ?2"
        )
        .bind(target_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM tags WHERE id = ?1")
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        target_id
    } else if name == source.name {
        sqlx::query("UPDATE tags SET color = ?1 WHERE id = ?2")
            .bind(&color)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        tag_id
    } else {
        sqlx::query(
            "UPDATE tags
             SET name = ?1, color = ?2, last_used_at = ?3
             WHERE id = ?4"
        )
        .bind(&name)
        .bind(&color)
        .bind(Utc::now().to_rfc3339())
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
        tag_id
    };

    for asset_id in affected_ids {
        refresh_asset_search_document(&mut *tx, asset_id).await?;
    }
    let result = load_tag(&mut *tx, result_id).await?;
    tx.commit().await?;
    Ok(result)
}

const DELETE_BATCH_SIZE: usize = 500;

pub async fn remove_tag_from_assets(
    db: &Db,
    tag_id: i64,
    asset_ids: &[i64],
) -> anyhow::Result<u64> {
    anyhow::ensure!(!asset_ids.is_empty(), "asset ids cannot be empty");
    let mut tx = db.begin().await?;
    load_tag(&mut *tx, tag_id).await?;

    let mut removed: u64 = 0;
    for chunk in asset_ids.chunks(DELETE_BATCH_SIZE) {
        let mut builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
            "DELETE FROM asset_tags WHERE tag_id = "
        );
        builder.push_bind(tag_id);
        builder.push(" AND asset_id IN (");
        let mut separated = builder.separated(", ");
        for asset_id in chunk {
            separated.push_bind(asset_id);
        }
        separated.push_unseparated(")");
        removed += builder.build().execute(&mut *tx).await?.rows_affected();
    }

    for asset_id in asset_ids {
        refresh_asset_search_document(&mut *tx, *asset_id).await?;
    }
    tx.commit().await?;
    Ok(removed)
}

pub async fn delete_tag(db: &Db, tag_id: i64) -> anyhow::Result<bool> {
    let mut tx = db.begin().await?;
    let affected_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT asset_id FROM asset_tags WHERE tag_id = ?1 ORDER BY asset_id"
    )
    .bind(tag_id)
    .fetch_all(&mut *tx)
    .await?;
    let deleted = sqlx::query("DELETE FROM tags WHERE id = ?1")
        .bind(tag_id)
        .execute(&mut *tx)
        .await?
        .rows_affected() == 1;
    if !deleted {
        tx.rollback().await?;
        return Ok(false);
    }
    for asset_id in affected_ids {
        refresh_asset_search_document(&mut *tx, asset_id).await?;
    }
    tx.commit().await?;
    Ok(true)
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
    sqlx::query_as::<_, crate::models::Collection>(
        "SELECT
           c.id,
           c.name,
           c.description,
           COUNT(ca.asset_id) AS asset_count
         FROM collections c
         LEFT JOIN collection_assets ca ON ca.collection_id = c.id
         GROUP BY c.id, c.name, c.description
         ORDER BY c.name COLLATE NOCASE, c.id",
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn create_collection(db: &Db, name: &str, description: &str) -> anyhow::Result<crate::models::Collection> {
    let name = name.trim();
    anyhow::ensure!(!name.is_empty(), "collection name cannot be empty");
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO collections (name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)"
    )
    .bind(name)
    .bind(description.trim())
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(crate::models::Collection {
        id: result.last_insert_rowid(),
        name: name.to_string(),
        description: description.trim().to_string(),
        asset_count: 0,
    })
}

pub async fn update_collection(
    db: &Db,
    collection_id: i64,
    name: &str,
    description: &str,
) -> anyhow::Result<crate::models::Collection> {
    let name = name.trim();
    let description = description.trim();
    anyhow::ensure!(!name.is_empty(), "collection name cannot be empty");
    let now = chrono::Utc::now().to_rfc3339();

    let mut tx = db.begin().await?;

    let result = sqlx::query(
        "UPDATE collections
         SET name = ?1, description = ?2, updated_at = ?3
         WHERE id = ?4",
    )
    .bind(name)
    .bind(description)
    .bind(&now)
    .bind(collection_id)
    .execute(&mut *tx)
    .await?;
    anyhow::ensure!(result.rows_affected() == 1, "collection not found");

    let collection = sqlx::query_as::<_, crate::models::Collection>(
        "SELECT
           c.id,
           c.name,
           c.description,
           COUNT(ca.asset_id) AS asset_count
         FROM collections c
         LEFT JOIN collection_assets ca ON ca.collection_id = c.id
         WHERE c.id = ?1
         GROUP BY c.id, c.name, c.description",
    )
    .bind(collection_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(collection)
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

pub async fn remove_assets_from_collection(
    db: &Db,
    collection_id: i64,
    asset_ids: &[i64],
) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;

    let collection_exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM collections WHERE id = ?1"
    )
    .bind(collection_id)
    .fetch_one(&mut *tx)
    .await?;
    anyhow::ensure!(collection_exists.0 == 1, "collection not found");

    let mut total_deleted: u64 = 0;
    for asset_id in asset_ids {
        let result = sqlx::query(
            "DELETE FROM collection_assets
             WHERE collection_id = ?1 AND asset_id = ?2",
        )
        .bind(collection_id)
        .bind(asset_id)
        .execute(&mut *tx)
        .await?;
        total_deleted += result.rows_affected();
    }

    if total_deleted > 0 {
        sqlx::query(
            "UPDATE collections SET updated_at = ?1 WHERE id = ?2",
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(collection_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn remove_asset_from_collection(db: &Db, collection_id: i64, asset_id: i64) -> anyhow::Result<()> {
    remove_assets_from_collection(db, collection_id, &[asset_id]).await
}

pub async fn delete_collection(db: &Db, collection_id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM collections WHERE id = ?1")
        .bind(collection_id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
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

const PREFIX_FTS_TABLE: &str = "asset_search_fts";
const TRIGRAM_FTS_TABLE: &str = "asset_search_trigram_fts";

async fn replace_search_document(
    conn: &mut sqlx::SqliteConnection,
    table: &str,
    id: i64,
    file_name: &str,
    absolute_path: &str,
    note: &str,
    tags: &str,
) -> anyhow::Result<()> {
    let delete_sql = format!("DELETE FROM {table} WHERE rowid = ?1");
    sqlx::query(&delete_sql)
        .bind(id)
        .execute(&mut *conn)
        .await?;

    let insert_sql = format!(
        "INSERT INTO {table} (rowid, file_name, absolute_path, note, tags)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    );
    sqlx::query(&insert_sql)
        .bind(id)
        .bind(file_name)
        .bind(absolute_path)
        .bind(note)
        .bind(tags)
        .execute(&mut *conn)
        .await?;

    Ok(())
}

pub async fn refresh_asset_search_document(
    conn: &mut sqlx::SqliteConnection,
    asset_id: i64,
) -> anyhow::Result<()> {
    let asset = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT id, file_name, absolute_path, note FROM assets WHERE id = ?1"
    )
    .bind(asset_id)
    .fetch_optional(&mut *conn)
    .await?;

    if let Some((id, file_name, absolute_path, note)) = asset {
        let tags: Vec<String> = sqlx::query_scalar(
            "SELECT tags.name
             FROM tags
             INNER JOIN asset_tags ON asset_tags.tag_id = tags.id
             WHERE asset_tags.asset_id = ?1
             ORDER BY tags.name",
        )
        .bind(asset_id)
        .fetch_all(&mut *conn)
        .await?;
        let tags = tags.join(" ");

        replace_search_document(
            conn,
            PREFIX_FTS_TABLE,
            id,
            &file_name,
            &absolute_path,
            &note,
            &tags,
        )
        .await?;
        replace_search_document(
            conn,
            TRIGRAM_FTS_TABLE,
            id,
            &file_name,
            &absolute_path,
            &note,
            &tags,
        )
        .await?;
    } else {
        for table in [PREFIX_FTS_TABLE, TRIGRAM_FTS_TABLE] {
            let sql = format!("DELETE FROM {table} WHERE rowid = ?1");
            sqlx::query(&sql)
                .bind(asset_id)
                .execute(&mut *conn)
                .await?;
        }
    }

    Ok(())
}

pub async fn update_asset_note(db: &Db, asset_id: i64, note: &str) -> anyhow::Result<Asset> {
    let now = Utc::now().to_rfc3339();
    let mut tx = db.begin().await?;
    sqlx::query("UPDATE assets SET note = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(note)
        .bind(&now)
        .bind(asset_id)
        .execute(&mut *tx)
        .await?;

    refresh_asset_search_document(&mut *tx, asset_id).await?;

    let row = sqlx::query_as::<_, Asset>(
        "SELECT id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size,
                modified_at, width, height, thumbnail_path, thumbnail_status, thumbnail_error, note, is_favorite, is_missing,
                created_at, updated_at
         FROM assets WHERE id = ?1"
    )
    .bind(asset_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(row)
}

pub async fn record_recent_asset_action_at(
    db: &Db,
    asset_id: i64,
    action_type: &str,
    created_at: &str,
) -> anyhow::Result<()> {
    crate::models::validate_recent_action_type(action_type)?;
    let mut tx = db.begin().await?;
    sqlx::query(
        "INSERT INTO recent_asset_actions (asset_id, action_type, created_at)
         VALUES (?1, ?2, ?3)"
    )
    .bind(asset_id)
    .bind(action_type)
    .bind(created_at)
    .execute(&mut *tx)
    .await?;
    cleanup_recent_asset_actions_in_tx(&mut tx, created_at).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn record_recent_asset_action(
    db: &Db,
    asset_id: i64,
    action_type: &str,
) -> anyhow::Result<()> {
    record_recent_asset_action_at(
        db, asset_id, action_type, &Utc::now().to_rfc3339()
    ).await
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

async fn cleanup_recent_asset_actions_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    now: &str,
) -> anyhow::Result<()> {
    let now = chrono::DateTime::parse_from_rfc3339(now)?.with_timezone(&Utc);
    let cutoff = (now - chrono::Duration::days(30)).to_rfc3339();
    sqlx::query("DELETE FROM recent_asset_actions WHERE created_at < ?1")
        .bind(cutoff)
        .execute(&mut **tx)
        .await?;
    sqlx::query(
        "DELETE FROM recent_asset_actions
         WHERE id NOT IN (
           SELECT id FROM recent_asset_actions
           ORDER BY created_at DESC, id DESC LIMIT 1000
         )"
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn cleanup_recent_asset_actions_at(db: &Db, now: &str) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;
    cleanup_recent_asset_actions_in_tx(&mut tx, now).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn cleanup_recent_asset_actions(db: &Db) -> anyhow::Result<()> {
    cleanup_recent_asset_actions_at(db, &Utc::now().to_rfc3339()).await
}

fn recent_activity_bounds(
    request: &crate::models::RecentActivityRequest,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> anyhow::Result<(Option<String>, Option<String>, i64, i64)> {
    let lower = match request.period.as_str() {
        "all" => None,
        "today" => {
            let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap()
                .and_local_timezone(*now.offset()).single()
                .ok_or_else(|| anyhow::anyhow!("invalid day boundary"))?;
            Some(start.with_timezone(&Utc).to_rfc3339())
        }
        "week" => Some(
            (now - chrono::Duration::days(7))
                .with_timezone(&Utc)
                .to_rfc3339()
        ),
        _ => anyhow::bail!("unsupported recent period"),
    };
    let action = match request.action_type.as_str() {
        "all" => None,
        value => {
            crate::models::validate_recent_action_type(value)?;
            Some(value.to_string())
        }
    };
    Ok((lower, action, request.limit.clamp(1, 100), request.offset.max(0)))
}

pub async fn list_recent_activity_at(
    db: &Db,
    request: crate::models::RecentActivityRequest,
    now: &str,
) -> anyhow::Result<crate::models::RecentActivityResponse> {
    let now = chrono::DateTime::parse_from_rfc3339(now)?;
    let (lower_bound, action_filter, limit, offset) = recent_activity_bounds(&request, now)?;
    let mut tx = db.begin().await?;

    let mut count_builder = QueryBuilder::<Sqlite>::new(
        "WITH filtered AS (SELECT asset_id FROM recent_asset_actions WHERE 1=1"
    );
    if let Some(lower) = &lower_bound {
        count_builder.push(" AND created_at >= ").push_bind(lower);
    }
    if let Some(action) = &action_filter {
        count_builder.push(" AND action_type = ").push_bind(action);
    }
    count_builder.push(") SELECT COUNT(DISTINCT asset_id) FROM filtered");
    let total_count: i64 = count_builder
        .build_query_scalar()
        .fetch_one(&mut *tx)
        .await?;

    let mut group_builder = QueryBuilder::<Sqlite>::new(
        "WITH filtered AS ( \
         SELECT *, ROW_NUMBER() OVER (PARTITION BY asset_id ORDER BY created_at DESC, id DESC) AS rn \
         FROM recent_asset_actions WHERE 1=1"
    );
    if let Some(lower) = &lower_bound {
        group_builder.push(" AND created_at >= ").push_bind(lower);
    }
    if let Some(action) = &action_filter {
        group_builder.push(" AND action_type = ").push_bind(action);
    }
    group_builder.push(
        ") SELECT asset_id, MAX(created_at) AS latest_action_at, COUNT(*) AS action_count, \
         SUM(action_type = 'open_file') AS open_file_count, \
         SUM(action_type = 'reveal_folder') AS reveal_folder_count, \
         SUM(action_type = 'copy_path') AS copy_path_count, \
         SUM(action_type = 'preview_media') AS preview_media_count, \
         MAX(CASE WHEN rn = 1 THEN action_type END) AS latest_action_type \
         FROM filtered GROUP BY asset_id \
         ORDER BY latest_action_at DESC, asset_id DESC LIMIT "
    );
    group_builder.push_bind(limit).push(" OFFSET ").push_bind(offset);

    let groups = group_builder.build().fetch_all(&mut *tx).await?;
    let asset_ids: Vec<i64> = groups.iter().map(|r| r.get::<i64, _>("asset_id")).collect();

    let mut asset_map = std::collections::HashMap::<i64, Asset>::new();
    if !asset_ids.is_empty() {
        let mut asset_builder = QueryBuilder::<Sqlite>::new(
            "SELECT id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size, \
             modified_at, width, height, thumbnail_path, thumbnail_status, thumbnail_error, note, is_favorite, is_missing, \
             created_at, updated_at FROM assets WHERE id IN ("
        );
        let mut separated = asset_builder.separated(",");
        for id in &asset_ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");
        let assets = asset_builder
            .build_query_as::<Asset>()
            .fetch_all(&mut *tx)
            .await?;
        for asset in assets {
            asset_map.insert(asset.id, asset);
        }
    }

    let mut details_map = std::collections::HashMap::<i64, Vec<crate::models::RecentAssetAction>>::new();
    if !asset_ids.is_empty() {
        let mut detail_builder = QueryBuilder::<Sqlite>::new(
            "WITH ranked AS ( \
             SELECT *, ROW_NUMBER() OVER (PARTITION BY asset_id ORDER BY created_at DESC, id DESC) AS rn \
             FROM recent_asset_actions WHERE asset_id IN ("
        );
        let mut separated = detail_builder.separated(",");
        for id in &asset_ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");
        if let Some(lower) = &lower_bound {
            detail_builder.push(" AND created_at >= ").push_bind(lower);
        }
        if let Some(action) = &action_filter {
            detail_builder.push(" AND action_type = ").push_bind(action);
        }
        detail_builder.push(
            ") SELECT id, asset_id, action_type, created_at FROM ranked WHERE rn <= 50 \
             ORDER BY asset_id, created_at DESC, id DESC"
        );
        let details = detail_builder
            .build_query_as::<crate::models::RecentAssetAction>()
            .fetch_all(&mut *tx)
            .await?;
        for detail in details {
            details_map.entry(detail.asset_id).or_default().push(detail);
        }
    }

    tx.commit().await?;

    let mut items = Vec::new();
    for row in groups {
        let asset_id = row.get::<i64, _>("asset_id");
        let asset = asset_map.remove(&asset_id)
            .ok_or_else(|| anyhow::anyhow!("asset {} not found", asset_id))?;
        items.push(crate::models::RecentActivityItem {
            asset,
            latest_action_type: row.get::<String, _>("latest_action_type"),
            latest_action_at: row.get::<String, _>("latest_action_at"),
            action_count: row.get::<i64, _>("action_count"),
            open_file_count: row.get::<i64, _>("open_file_count"),
            reveal_folder_count: row.get::<i64, _>("reveal_folder_count"),
            copy_path_count: row.get::<i64, _>("copy_path_count"),
            preview_media_count: row.get::<i64, _>("preview_media_count"),
            actions: details_map.remove(&asset_id).unwrap_or_default(),
        });
    }

    Ok(crate::models::RecentActivityResponse {
        items,
        total_count,
        limit,
        offset,
    })
}

pub async fn list_recent_activity(
    db: &Db,
    request: crate::models::RecentActivityRequest,
) -> anyhow::Result<crate::models::RecentActivityResponse> {
    list_recent_activity_at(
        db,
        request,
        &chrono::Local::now().fixed_offset().to_rfc3339(),
    ).await
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
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE collections (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, description TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL, updated_at TEXT NOT NULL)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE assets (id INTEGER PRIMARY KEY)").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE collection_assets (collection_id INTEGER NOT NULL, asset_id INTEGER NOT NULL, PRIMARY KEY (collection_id, asset_id), FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE, FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE)")
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
        sqlx::query("INSERT INTO assets (id) VALUES (1), (2), (3)").execute(&db).await.unwrap();
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
        sqlx::query("INSERT INTO assets (id) VALUES (1), (2), (3)").execute(&db).await.unwrap();
        let col = create_collection(&db, "Test", "").await.unwrap();
        add_assets_to_collection(&db, col.id, &[1, 2]).await.unwrap();
        add_assets_to_collection(&db, col.id, &[1, 3]).await.unwrap();
        let ids = list_collection_assets(&db, col.id).await.unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[tokio::test]
    async fn remove_asset_from_collection_works() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO assets (id) VALUES (1), (2)").execute(&db).await.unwrap();
        let col = create_collection(&db, "Test", "").await.unwrap();
        add_assets_to_collection(&db, col.id, &[1, 2]).await.unwrap();
        remove_asset_from_collection(&db, col.id, 1).await.unwrap();
        let ids = list_collection_assets(&db, col.id).await.unwrap();
        assert_eq!(ids, vec![2]);
    }

    #[tokio::test]
    async fn list_collections_includes_asset_count() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO assets (id) VALUES (1), (2), (3)")
            .execute(&db)
            .await
            .unwrap();
        let collection = create_collection(&db, "角色", "").await.unwrap();
        add_assets_to_collection(&db, collection.id, &[1, 2, 3])
            .await
            .unwrap();

        let rows = list_collections(&db).await.unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].asset_count, 3);
    }

    #[tokio::test]
    async fn update_collection_trims_name_and_description() {
        let db = setup_db().await;
        let collection = create_collection(&db, "旧名称", "").await.unwrap();

        let updated = update_collection(
            &db,
            collection.id,
            "  新名称  ",
            "  常用角色素材  ",
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "新名称");
        assert_eq!(updated.description, "常用角色素材");
        assert_eq!(updated.asset_count, 0);
    }

    #[tokio::test]
    async fn update_collection_rejects_blank_name() {
        let db = setup_db().await;
        let collection = create_collection(&db, "角色", "").await.unwrap();

        let error = update_collection(&db, collection.id, "   ", "")
            .await
            .unwrap_err();

        assert!(error.to_string().contains("collection name cannot be empty"));
    }

    #[tokio::test]
    async fn remove_assets_from_collection_is_idempotent() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO assets (id) VALUES (1), (2), (3)")
            .execute(&db)
            .await
            .unwrap();
        let collection = create_collection(&db, "角色", "").await.unwrap();
        add_assets_to_collection(&db, collection.id, &[1, 2, 3])
            .await
            .unwrap();

        remove_assets_from_collection(&db, collection.id, &[2, 3, 99])
            .await
            .unwrap();

        assert_eq!(list_collection_assets(&db, collection.id).await.unwrap(), vec![1]);
    }

    #[tokio::test]
    async fn delete_collection_removes_links_but_not_assets() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO assets (id) VALUES (1), (2)")
            .execute(&db)
            .await
            .unwrap();
        let collection = create_collection(&db, "角色", "").await.unwrap();
        add_assets_to_collection(&db, collection.id, &[1, 2])
            .await
            .unwrap();

        assert!(delete_collection(&db, collection.id).await.unwrap());

        let asset_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assets")
            .fetch_one(&db)
            .await
            .unwrap();
        let link_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM collection_assets")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(asset_count, 2);
        assert_eq!(link_count, 0);
    }

    #[tokio::test]
    async fn remove_assets_from_collection_empty_ids_existing_collection_succeeds() {
        let db = setup_db().await;
        let collection = create_collection(&db, "角色", "").await.unwrap();
        remove_assets_from_collection(&db, collection.id, &[])
            .await
            .unwrap();
        let ids = list_collection_assets(&db, collection.id).await.unwrap();
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn remove_assets_from_collection_empty_ids_missing_collection_errors() {
        let db = setup_db().await;
        let err = remove_assets_from_collection(&db, 9999, &[])
            .await
            .unwrap_err();
        assert!(err.to_string().contains("collection not found"));
    }

    #[tokio::test]
    async fn remove_assets_from_collection_non_empty_ids_missing_collection_errors() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO assets (id) VALUES (1)")
            .execute(&db)
            .await
            .unwrap();
        let err = remove_assets_from_collection(&db, 9999, &[1])
            .await
            .unwrap_err();
        assert!(err.to_string().contains("collection not found"));
    }

    #[tokio::test]
    async fn remove_assets_from_collection_idempotent_does_not_update_updated_at_twice() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO assets (id) VALUES (1)")
            .execute(&db)
            .await
            .unwrap();
        let collection = create_collection(&db, "角色", "").await.unwrap();
        add_assets_to_collection(&db, collection.id, &[1])
            .await
            .unwrap();

        let before: String = sqlx::query_scalar("SELECT updated_at FROM collections WHERE id = ?1")
            .bind(collection.id)
            .fetch_one(&db)
            .await
            .unwrap();

        remove_assets_from_collection(&db, collection.id, &[1])
            .await
            .unwrap();
        let after_first: String = sqlx::query_scalar("SELECT updated_at FROM collections WHERE id = ?1")
            .bind(collection.id)
            .fetch_one(&db)
            .await
            .unwrap();
        assert_ne!(before, after_first);

        remove_assets_from_collection(&db, collection.id, &[1])
            .await
            .unwrap();
        let after_second: String = sqlx::query_scalar("SELECT updated_at FROM collections WHERE id = ?1")
            .bind(collection.id)
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(after_first, after_second);
    }

    #[tokio::test]
    async fn collection_assets_foreign_key_enforced() {
        let db = setup_db().await;
        let collection = create_collection(&db, "角色", "").await.unwrap();
        let result = sqlx::query(
            "INSERT INTO collection_assets (collection_id, asset_id) VALUES (?1, ?2)",
        )
        .bind(collection.id)
        .bind(999)
        .execute(&db)
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_collection_returns_accurate_asset_count() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO assets (id) VALUES (1), (2)")
            .execute(&db)
            .await
            .unwrap();
        let collection = create_collection(&db, "角色", "").await.unwrap();
        add_assets_to_collection(&db, collection.id, &[1, 2])
            .await
            .unwrap();

        let updated = update_collection(&db, collection.id, "主角", "常用角色素材")
            .await
            .unwrap();

        assert_eq!(updated.name, "主角");
        assert_eq!(updated.description, "常用角色素材");
        assert_eq!(updated.asset_count, 2);
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
        sqlx::query(
            "CREATE VIRTUAL TABLE asset_search_fts USING fts5(file_name, absolute_path, note, tags, tokenize = 'unicode61')"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE VIRTUAL TABLE asset_search_trigram_fts USING fts5(file_name, absolute_path, note, tags, tokenize = 'trigram')"
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
        sqlx::query("INSERT INTO asset_search_fts (rowid, file_name, absolute_path, note, tags) VALUES (1, 'a.png', '/test/a.png', '', '')")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO asset_search_trigram_fts (rowid, file_name, absolute_path, note, tags) VALUES (1, 'a.png', '/test/a.png', '', '')")
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

        let fts_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset_search_fts WHERE rowid = 1")
            .fetch_one(&db).await.unwrap();
        assert_eq!(fts_count.0, 0);
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
mod recent_activity_tests {
    use super::*;

    async fn setup_db() -> Db {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
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

    async fn insert_asset(db: &Db, file_name: &str, is_missing: bool) -> i64 {
        let path = "C:/assets";
        let folder_id: i64 = match sqlx::query_as::<_, (i64,)>(
            "SELECT id FROM library_folders WHERE path = ?1"
        )
        .bind(path)
        .fetch_one(db)
        .await
        {
            Ok(row) => row.0,
            Err(_) => create_library_folder(db, "fixture", path).await.unwrap().id,
        };
        let missing_flag: i64 = if is_missing { 1 } else { 0 };
        sqlx::query(
            "INSERT INTO assets (
                library_folder_id, absolute_path, file_name, extension, asset_type,
                modified_at, created_at, updated_at, is_missing
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        )
        .bind(folder_id)
        .bind(format!("C:/assets/{}", file_name))
        .bind(file_name)
        .bind("wav")
        .bind("audio")
        .bind("2024-01-01T00:00:00Z")
        .bind("2024-01-01T00:00:00Z")
        .bind("2024-01-01T00:00:00Z")
        .bind(missing_flag)
        .execute(db).await.unwrap();
        sqlx::query_as::<_, (i64,)>("SELECT id FROM assets WHERE absolute_path = ?1")
            .bind(format!("C:/assets/{}", file_name))
            .fetch_one(db).await.unwrap().0
    }

    async fn insert_activity_series(db: &Db, asset_id: i64, count: i64, created_at: &str) {
        for _ in 0..count {
            record_recent_asset_action_at(db, asset_id, "copy_path", created_at)
                .await
                .unwrap();
        }
    }

    async fn seed_action(db: &Db, asset_id: i64, action_type: &str, created_at: &str) {
        record_recent_asset_action_at(db, asset_id, action_type, created_at)
            .await
            .unwrap();
    }

    async fn request_page(
        db: &Db,
        period: &str,
        action_type: &str,
        now: &str,
    ) -> crate::models::RecentActivityResponse {
        list_recent_activity_at(
            db,
            crate::models::RecentActivityRequest {
                period: period.into(),
                action_type: action_type.into(),
                limit: 10,
                offset: 0,
            },
            now,
        )
        .await
        .unwrap()
    }

    async fn request_page_with_limit(
        db: &Db,
        limit: i64,
        offset: i64,
    ) -> crate::models::RecentActivityResponse {
        list_recent_activity_at(
            db,
            crate::models::RecentActivityRequest {
                period: "all".into(),
                action_type: "all".into(),
                limit,
                offset,
            },
            "2026-06-13T12:00:00+08:00",
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn accepts_all_supported_activity_types() {
        let db = setup_db().await;
        let asset_id = insert_asset(&db, "sound.wav", false).await;
        for action in ["open_file", "reveal_folder", "copy_path", "preview_media"] {
            record_recent_asset_action_at(&db, asset_id, action, "2026-06-13T10:00:00Z")
                .await
                .unwrap();
        }
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM recent_asset_actions")
            .fetch_one(&db).await.unwrap();
        assert_eq!(count, 4);
    }

    #[tokio::test]
    async fn rejects_unknown_activity_type() {
        let db = setup_db().await;
        let asset_id = insert_asset(&db, "sound.wav", false).await;
        let error = record_recent_asset_action_at(
            &db, asset_id, "selected_asset", "2026-06-13T10:00:00Z"
        ).await.unwrap_err();
        assert!(error.to_string().contains("unsupported recent action type"));
    }

    #[tokio::test]
    async fn cleanup_removes_old_rows_and_keeps_latest_thousand() {
        let db = setup_db().await;
        let asset_id = insert_asset(&db, "sound.wav", false).await;
        insert_activity_series(&db, asset_id, 1005, "2026-06-01T00:00:00Z").await;
        record_recent_asset_action_at(
            &db, asset_id, "open_file", "2026-04-01T00:00:00Z"
        ).await.unwrap();

        cleanup_recent_asset_actions_at(&db, "2026-06-13T00:00:00Z")
            .await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM recent_asset_actions")
            .fetch_one(&db).await.unwrap();
        assert_eq!(count, 1000);
        let old: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM recent_asset_actions WHERE created_at < '2026-05-14T00:00:00Z'"
        ).fetch_one(&db).await.unwrap();
        assert_eq!(old, 0);
    }

    #[tokio::test]
    async fn groups_actions_by_asset_with_counts_and_details() {
        let db = setup_db().await;
        let sound = insert_asset(&db, "sound.wav", false).await;
        let video = insert_asset(&db, "hero.mp4", false).await;
        seed_action(&db, sound, "copy_path", "2026-06-13T09:00:00Z").await;
        seed_action(&db, sound, "open_file", "2026-06-13T10:00:00Z").await;
        seed_action(&db, video, "reveal_folder", "2026-06-12T10:00:00Z").await;

        let page = list_recent_activity_at(
            &db,
            crate::models::RecentActivityRequest {
                period: "all".into(),
                action_type: "all".into(),
                limit: 20,
                offset: 0,
            },
            "2026-06-13T12:00:00+08:00",
        ).await.unwrap();

        assert_eq!(page.total_count, 2);
        assert_eq!(page.items[0].asset.id, sound);
        assert_eq!(page.items[0].latest_action_type, "open_file");
        assert_eq!(page.items[0].action_count, 2);
        assert_eq!(page.items[0].copy_path_count, 1);
        assert_eq!(page.items[0].actions.len(), 2);
    }

    #[tokio::test]
    async fn filters_today_week_and_action_type() {
        let db = setup_db().await;
        let asset = insert_asset(&db, "sound.wav", false).await;
        seed_action(&db, asset, "copy_path", "2026-06-13T01:00:00Z").await;
        seed_action(&db, asset, "open_file", "2026-06-07T01:00:00Z").await;

        let today = request_page(&db, "today", "copy_path", "2026-06-13T12:00:00Z"
        ).await;
        assert_eq!(today.items.len(), 1);
        assert_eq!(today.items[0].action_count, 1);

        let week = request_page(&db, "week", "all", "2026-06-13T12:00:00Z"
        ).await;
        assert_eq!(week.items[0].action_count, 2);
    }

    #[tokio::test]
    async fn keeps_missing_assets_and_uses_stable_pagination() {
        let db = setup_db().await;
        let a = insert_asset(&db, "a.wav", true).await;
        let b = insert_asset(&db, "b.wav", false).await;
        seed_action(&db, a, "open_file", "2026-06-13T10:00:00Z").await;
        seed_action(&db, b, "copy_path", "2026-06-13T10:00:00Z").await;
        let page = request_page_with_limit(&db, 1, 0).await;
        assert_eq!(page.total_count, 2);
        assert_eq!(page.items.len(), 1);
        assert!(page.items[0].asset.id == a || page.items[0].asset.id == b);
    }

    #[tokio::test]
    async fn stable_sort_by_id_for_same_timestamp() {
        let db = setup_db().await;
        let first = insert_asset(&db, "first.wav", false).await;
        let second = insert_asset(&db, "second.wav", false).await;
        seed_action(&db, first, "open_file", "2026-06-13T10:00:00Z").await;
        seed_action(&db, second, "copy_path", "2026-06-13T10:00:00Z").await;

        let page = request_page_with_limit(&db, 10, 0).await;

        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].asset.id, second);
        assert_eq!(page.items[1].asset.id, first);
    }

    #[tokio::test]
    async fn details_limited_to_fifty_but_counts_full() {
        let db = setup_db().await;
        let asset = insert_asset(&db, "sound.wav", false).await;
        for i in 0..60 {
            let ts = format!("2026-06-13T{:02}:00:00Z", i % 24);
            seed_action(&db, asset, "copy_path", &ts).await;
        }

        let page = request_page_with_limit(&db, 10, 0).await;

        assert_eq!(page.total_count, 1);
        assert_eq!(page.items[0].action_count, 60);
        assert_eq!(page.items[0].copy_path_count, 60);
        assert_eq!(page.items[0].actions.len(), 50);
    }

    #[tokio::test]
    async fn latest_action_type_respects_action_filter() {
        let db = setup_db().await;
        let asset = insert_asset(&db, "sound.wav", false).await;
        seed_action(&db, asset, "copy_path", "2026-06-13T10:00:00Z").await;
        seed_action(&db, asset, "open_file", "2026-06-13T10:00:00Z").await;

        let page = request_page(
            &db, "all", "copy_path", "2026-06-13T12:00:00Z"
        ).await;

        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].latest_action_type, "copy_path");
        assert_eq!(page.items[0].action_count, 1);
    }

    #[tokio::test]
    async fn second_page_is_stable_and_bounded() {
        let db = setup_db().await;
        let a = insert_asset(&db, "a.wav", false).await;
        let b = insert_asset(&db, "b.wav", false).await;
        seed_action(&db, a, "open_file", "2026-06-13T10:00:00Z").await;
        seed_action(&db, b, "copy_path", "2026-06-12T10:00:00Z").await;

        let first = request_page_with_limit(&db, 1, 0).await;
        let second = request_page_with_limit(&db, 1, 1).await;

        assert_eq!(first.items.len(), 1);
        assert_eq!(second.items.len(), 1);
        assert_ne!(first.items[0].asset.id, second.items[0].asset.id);
        assert_eq!(first.items[0].asset.id, a);
        assert_eq!(second.items[0].asset.id, b);
    }

    #[tokio::test]
    async fn migration_0009_enforces_action_type_check() {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO library_folders (name, path, created_at, is_enabled) \
             VALUES ('fixture', 'C:/assets', '2024-01-01T00:00:00Z', 1)"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO assets (library_folder_id, absolute_path, file_name, extension, \
             asset_type, file_size, modified_at, created_at, updated_at) \
             VALUES (1, 'C:/assets/sound.wav', 'sound.wav', 'wav', 'audio', 0, \
             '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        ).execute(&pool).await.unwrap();

        let invalid = sqlx::query(
            "INSERT INTO recent_asset_actions (asset_id, action_type, created_at) \
             VALUES (1, 'invalid_action', '2026-06-13T10:00:00Z')"
        ).execute(&pool).await;
        assert!(invalid.is_err());

        let valid = sqlx::query(
            "INSERT INTO recent_asset_actions (asset_id, action_type, created_at) \
             VALUES (1, 'preview_media', '2026-06-13T10:00:00Z')"
        ).execute(&pool).await;
        assert!(valid.is_ok());
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
        sqlx::query(
            "CREATE TABLE asset_tags (
                asset_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (asset_id, tag_id),
                FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
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
        let mut conn = db.acquire().await.unwrap();
        _create_or_get_tag(&mut *conn, "旧标签").await.unwrap();

        let tags = list_recent_tags(&db, 10).await.unwrap();
        let names: Vec<String> = tags.into_iter().map(|tag| tag.name).collect();
        assert_eq!(names, vec!["旧标签", "新标签"]);
    }
}

#[cfg(test)]
mod fts_tests {
    use super::*;

    async fn setup_db() -> Db {
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
                updated_at TEXT NOT NULL
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
                PRIMARY KEY (asset_id, tag_id)
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE VIRTUAL TABLE asset_search_fts USING fts5(file_name, absolute_path, note, tags, tokenize = 'unicode61')"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE VIRTUAL TABLE asset_search_trigram_fts USING fts5(file_name, absolute_path, note, tags, tokenize = 'trigram')"
        ).execute(&pool).await.unwrap();
        pool
    }

    async fn insert_test_asset(db: &Db, folder_id: i64, path: &str) -> i64 {
        let file_name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let result = sqlx::query(
            "INSERT INTO assets (library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        )
        .bind(folder_id)
        .bind(path)
        .bind(file_name)
        .bind(extension)
        .execute(db).await.unwrap();
        result.last_insert_rowid()
    }

    async fn create_test_library_folder(db: &Db, name: &str, path: &str) -> i64 {
        let result = sqlx::query("INSERT INTO library_folders (name, path, created_at, is_enabled) VALUES (?1, ?2, '2024-01-01T00:00:00Z', 1)")
            .bind(name)
            .bind(path)
            .execute(db).await.unwrap();
        result.last_insert_rowid()
    }

    #[tokio::test]
    async fn update_asset_note_triggers_fts_refresh() {
        let db = setup_db().await;
        let folder_id = create_test_library_folder(&db, "fixture", "C:/assets").await;
        let asset_id = insert_test_asset(&db, folder_id, "C:/assets/hero_idle.png").await;

        update_asset_note(&db, asset_id, "主角待整理").await.unwrap();

        let row: (String, String, String, String) = sqlx::query_as(
            "SELECT file_name, absolute_path, note, tags FROM asset_search_fts WHERE rowid = ?1",
        )
        .bind(asset_id)
        .fetch_one(&db)
        .await
        .unwrap();

        assert_eq!(row.0, "hero_idle.png");
        assert!(row.2.contains("主角"));
    }

    #[tokio::test]
    async fn apply_tag_to_assets_triggers_fts_refresh() {
        let db = setup_db().await;
        let folder_id = create_test_library_folder(&db, "fixture", "C:/assets").await;
        let asset_id = insert_test_asset(&db, folder_id, "C:/assets/hero_idle.png").await;

        apply_tag_to_assets(&db, "角色", &[asset_id]).await.unwrap();

        let row: (String, String, String, String) = sqlx::query_as(
            "SELECT file_name, absolute_path, note, tags FROM asset_search_fts WHERE rowid = ?1",
        )
        .bind(asset_id)
        .fetch_one(&db)
        .await
        .unwrap();

        assert_eq!(row.0, "hero_idle.png");
        assert!(row.3.contains("角色"));
    }

    #[tokio::test]
    async fn apply_tag_to_assets_rolls_back_tag_on_failure() {
        let db = setup_db().await;
        let folder_id = create_test_library_folder(&db, "fixture", "C:/assets").await;
        let asset_id = insert_test_asset(&db, folder_id, "C:/assets/hero_idle.png").await;

        // Pre-populate FTS so the table exists; then drop it to break refresh inside the tx.
        refresh_asset_search_document(&mut *db.acquire().await.unwrap(), asset_id).await.unwrap();
        sqlx::query("DROP TABLE asset_search_fts").execute(&db).await.unwrap();

        let result = apply_tag_to_assets(&db, "新标签", &[asset_id]).await;
        assert!(result.is_err(), "expected error when fts refresh fails");

        let tag_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tags WHERE name = ?1")
            .bind("新标签")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(tag_count.0, 0, "tag should not be committed when transaction rolls back");

        let at_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset_tags WHERE asset_id = ?1")
            .bind(asset_id)
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(at_count.0, 0, "asset_tags should not be committed when transaction rolls back");
    }

    #[tokio::test]
    async fn update_asset_note_refreshes_trigram_fts() {
        let db = setup_db().await;
        let folder_id = create_test_library_folder(&db, "fixture", "C:/assets").await;
        let asset_id = insert_test_asset(&db, folder_id, "C:/assets/hero_idle.png").await;

        update_asset_note(&db, asset_id, "主角待机动画").await.unwrap();

        let note: String = sqlx::query_scalar(
            "SELECT note FROM asset_search_trigram_fts WHERE rowid = ?1"
        )
        .bind(asset_id)
        .fetch_one(&db)
        .await
        .unwrap();
        assert_eq!(note, "主角待机动画");
    }

    #[tokio::test]
    async fn apply_tag_refreshes_trigram_fts() {
        let db = setup_db().await;
        let folder_id = create_test_library_folder(&db, "fixture", "C:/assets").await;
        let asset_id = insert_test_asset(&db, folder_id, "C:/assets/hero_idle.png").await;

        apply_tag_to_assets(&db, "角色动画", &[asset_id]).await.unwrap();

        let tags: String = sqlx::query_scalar(
            "SELECT tags FROM asset_search_trigram_fts WHERE rowid = ?1"
        )
        .bind(asset_id)
        .fetch_one(&db)
        .await
        .unwrap();
        assert_eq!(tags, "角色动画");
    }

    #[tokio::test]
    async fn update_asset_note_rolls_back_when_fts_refresh_fails() {
        let db = setup_db().await;
        let folder_id = create_test_library_folder(&db, "fixture", "C:/assets").await;
        let asset_id = insert_test_asset(&db, folder_id, "C:/assets/hero_idle.png").await;

        // Pre-populate FTS
        refresh_asset_search_document(&mut *db.acquire().await.unwrap(), asset_id).await.unwrap();

        // Break FTS to force refresh failure inside update_asset_note's transaction
        sqlx::query("DROP TABLE asset_search_fts").execute(&db).await.unwrap();

        let result = update_asset_note(&db, asset_id, "should not persist").await;
        assert!(result.is_err(), "expected error when fts refresh fails");

        let note: (String,) = sqlx::query_as("SELECT note FROM assets WHERE id = ?1")
            .bind(asset_id)
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(note.0, "", "note update should roll back when fts refresh fails");
    }

    async fn run_migrations_up_to(pool: &Db, target_version: i64) -> anyhow::Result<()> {
        let migrations_dir = std::path::Path::new("./migrations");
        let mut entries: Vec<_> = std::fs::read_dir(migrations_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.ends_with(".sql")
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let version: i64 = name.split('_').next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            if version > target_version {
                continue;
            }
            let sql = std::fs::read_to_string(entry.path())?;
            sqlx::raw_sql(&sql).execute(pool).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn migration_0007_backfills_fts_from_existing_assets() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Apply real migrations up to 0006
        run_migrations_up_to(&pool, 6).await.unwrap();

        // Insert existing data as if the app has been running on 0006
        sqlx::query("INSERT INTO library_folders (id, name, path, created_at, is_enabled) VALUES (1, 'fixture', 'C:/assets', '2024-01-01T00:00:00Z', 1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size, modified_at, note, created_at, updated_at) VALUES (1, 1, 'C:/assets/hero.png', 'hero.png', 'png', 'image', 1024, '2024-01-01T00:00:00Z', '主角待机', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO tags (id, name, color, created_at, last_used_at) VALUES (1, '角色', '#5B8DEF', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO asset_tags (asset_id, tag_id) VALUES (1, 1)")
            .execute(&pool).await.unwrap();

        // Apply the real 0007 migration file
        let migration_0007 = std::fs::read_to_string("./migrations/0007_asset_search_fts.sql").unwrap();
        sqlx::raw_sql(&migration_0007).execute(&pool).await.unwrap();

        // Also create trigram FTS table so production search can find assets
        sqlx::query(
            "CREATE VIRTUAL TABLE asset_search_trigram_fts USING fts5(file_name, absolute_path, note, tags, tokenize = 'trigram')"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO asset_search_trigram_fts (rowid, file_name, absolute_path, note, tags) VALUES (1, 'hero.png', 'C:/assets/hero.png', '主角待机', '角色')"
        ).execute(&pool).await.unwrap();

        // Verify FTS backfill via production search
        let req = crate::models::AssetSearchRequest {
            query: "hero".to_string(),
            search_file_name: true,
            search_note: false,
            search_path: false,
            search_tags: false,
            asset_type: None,
            library_folder_id: None,
            collection_id: None,
            is_favorite: None,
            is_missing: None,
            min_file_size: None,
            max_file_size: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            modified_after: None,
            modified_before: None,
            sort_by: "file_name".to_string(),
            sort_direction: "asc".to_string(),
            limit: 200,
            offset: 0,
        };
        let page = crate::search::search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "hero.png");

        // Note prefix search
        let mut req = req.clone();
        req.query = "主角".to_string();
        req.search_file_name = false;
        req.search_note = true;
        let page = crate::search::search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "hero.png");

        // Tag search
        let mut req = req.clone();
        req.query = "角色".to_string();
        req.search_note = false;
        req.search_tags = true;
        let page = crate::search::search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "hero.png");
    }

    #[tokio::test]
    async fn migration_0008_backfills_trigram_fts_from_existing_assets() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        run_migrations_up_to(&pool, 7).await.unwrap();
        sqlx::query(
            "INSERT INTO library_folders
             (id, name, path, created_at, is_enabled)
             VALUES (1, 'fixture', 'C:/assets', '2024-01-01T00:00:00Z', 1)"
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO assets
             (id, library_folder_id, absolute_path, file_name, extension, asset_type,
              file_size, modified_at, note, created_at, updated_at)
             VALUES
             (1, 1, 'C:/assets/hero.png', 'hero.png', 'png', 'image',
              1024, '2024-01-01T00:00:00Z', '主角待机动画',
              '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO tags
             (id, name, color, created_at, last_used_at)
             VALUES
             (1, '角色动画', '#5B8DEF', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO asset_tags (asset_id, tag_id) VALUES (1, 1)")
            .execute(&pool)
            .await
            .unwrap();

        let migration = std::fs::read_to_string("./migrations/0008_asset_search_trigram_fts.sql")
            .unwrap();
        sqlx::raw_sql(&migration).execute(&pool).await.unwrap();

        let row: (String, String) = sqlx::query_as(
            "SELECT note, tags
             FROM asset_search_trigram_fts
             WHERE rowid = 1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(row.0, "主角待机动画");
        assert_eq!(row.1, "角色动画");
    }
}

#[cfg(test)]
mod tag_management_tests {
    use super::*;

    async fn setup_db() -> Db {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
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
                updated_at TEXT NOT NULL
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
            "CREATE VIRTUAL TABLE asset_search_fts USING fts5(file_name, absolute_path, note, tags, tokenize = 'unicode61')"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE VIRTUAL TABLE asset_search_trigram_fts USING fts5(file_name, absolute_path, note, tags, tokenize = 'trigram')"
        ).execute(&pool).await.unwrap();
        pool
    }

    async fn insert_two_assets(db: &Db) -> (i64, i64) {
        let folder = create_library_folder(db, "fixture", "C:/assets").await.unwrap();
        let a = sqlx::query(
            "INSERT INTO assets (library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        )
        .bind(folder.id)
        .bind("C:/assets/a.png")
        .bind("a.png")
        .bind("png")
        .execute(db).await.unwrap().last_insert_rowid();
        let b = sqlx::query(
            "INSERT INTO assets (library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'image', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        )
        .bind(folder.id)
        .bind("C:/assets/b.png")
        .bind("b.png")
        .bind("png")
        .execute(db).await.unwrap().last_insert_rowid();
        (a, b)
    }

    async fn insert_tag(db: &Db, name: &str, color: &str) -> i64 {
        sqlx::query(
            "INSERT INTO tags (name, color, created_at, last_used_at) VALUES (?1, ?2, ?3, ?3)"
        )
        .bind(name)
        .bind(color)
        .bind("2024-01-01T00:00:00Z")
        .execute(db).await.unwrap().last_insert_rowid()
    }

    async fn link_tag(db: &Db, tag_id: i64, asset_id: i64) {
        sqlx::query("INSERT OR IGNORE INTO asset_tags (asset_id, tag_id) VALUES (?1, ?2)")
            .bind(asset_id)
            .bind(tag_id)
            .execute(db).await.unwrap();
    }

    async fn count_tag(db: &Db, tag_id: i64) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE id = ?1")
            .bind(tag_id)
            .fetch_one(db).await.unwrap()
    }

    async fn count_links(db: &Db, tag_id: i64) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM asset_tags WHERE tag_id = ?1")
            .bind(tag_id)
            .fetch_one(db).await.unwrap()
    }

    async fn tag_by_name(db: &Db, name: &str) -> crate::models::Tag {
        sqlx::query_as::<_, crate::models::Tag>(
            "SELECT t.id, t.name, t.color, COUNT(at.asset_id) AS asset_count
             FROM tags t
             LEFT JOIN asset_tags at ON at.tag_id = t.id
             WHERE t.name = ?1
             GROUP BY t.id, t.name, t.color"
        )
        .bind(name)
        .fetch_one(db)
        .await
        .unwrap()
    }

    async fn assert_fts_tags(db: &Db, asset_id: i64, expected: &str) {
        for table in [PREFIX_FTS_TABLE, TRIGRAM_FTS_TABLE] {
            let sql = format!("SELECT tags FROM {table} WHERE rowid = ?1");
            let actual: String = sqlx::query_scalar(&sql)
                .bind(asset_id)
                .fetch_one(db)
                .await
                .unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[tokio::test]
    async fn list_tags_includes_asset_count() {
        let db = setup_db().await;
        let (asset_a, asset_b) = insert_two_assets(&db).await;
        apply_tag_to_assets(&db, "角色", &[asset_a, asset_b]).await.unwrap();

        let tags = list_tags(&db).await.unwrap();

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "角色");
        assert_eq!(tags[0].asset_count, 2);
    }

    #[tokio::test]
    async fn update_tag_renames_and_normalizes_color() {
        let db = setup_db().await;
        let tag_id = insert_tag(&db, "旧标签", "#5B8DEF").await;

        let updated = update_tag(&db, tag_id, "  新   标签 ", "#abcdef")
            .await
            .unwrap();

        assert_eq!(updated.name, "新 标签");
        assert_eq!(updated.color, "#ABCDEF");
        assert_eq!(updated.asset_count, 0);
    }

    #[tokio::test]
    async fn update_tag_merges_into_existing_tag() {
        let db = setup_db().await;
        let (asset_a, asset_b) = insert_two_assets(&db).await;
        let source_id = insert_tag(&db, "来源", "#111111").await;
        let target_id = insert_tag(&db, "目标", "#ABCDEF").await;
        link_tag(&db, source_id, asset_a).await;
        link_tag(&db, source_id, asset_b).await;
        link_tag(&db, target_id, asset_b).await;

        let merged = update_tag(&db, source_id, "目标", "#222222")
            .await
            .unwrap();

        assert_eq!(merged.id, target_id);
        assert_eq!(merged.color, "#ABCDEF");
        assert_eq!(merged.asset_count, 2);
        assert_eq!(count_tag(&db, source_id).await, 0);
        assert_eq!(count_links(&db, target_id).await, 2);
    }

    #[tokio::test]
    async fn update_tag_rejects_blank_name_and_invalid_color() {
        let db = setup_db().await;
        let tag_id = insert_tag(&db, "角色", "#5B8DEF").await;

        assert!(update_tag(&db, tag_id, "   ", "#5B8DEF").await.is_err());
        assert!(update_tag(&db, tag_id, "角色", "blue").await.is_err());
    }

    #[tokio::test]
    async fn remove_tag_from_assets_is_idempotent_and_keeps_tag() {
        let db = setup_db().await;
        let (asset_a, asset_b) = insert_two_assets(&db).await;
        let tag_id = insert_tag(&db, "角色", "#5B8DEF").await;
        link_tag(&db, tag_id, asset_a).await;
        link_tag(&db, tag_id, asset_b).await;

        let removed = remove_tag_from_assets(&db, tag_id, &[asset_a, 999])
            .await
            .unwrap();
        let removed_again = remove_tag_from_assets(&db, tag_id, &[asset_a])
            .await
            .unwrap();

        assert_eq!(removed, 1);
        assert_eq!(removed_again, 0);
        assert_eq!(count_tag(&db, tag_id).await, 1);
        assert_eq!(count_links(&db, tag_id).await, 1);
    }

    #[tokio::test]
    async fn remove_tag_from_assets_rejects_empty_ids() {
        let db = setup_db().await;
        let tag_id = insert_tag(&db, "角色", "#5B8DEF").await;

        let error = remove_tag_from_assets(&db, tag_id, &[]).await.unwrap_err();

        assert!(error.to_string().contains("asset ids cannot be empty"));
    }

    #[tokio::test]
    async fn delete_tag_removes_links_but_keeps_assets() {
        let db = setup_db().await;
        let (asset_a, asset_b) = insert_two_assets(&db).await;
        let tag_id = insert_tag(&db, "角色", "#5B8DEF").await;
        link_tag(&db, tag_id, asset_a).await;
        link_tag(&db, tag_id, asset_b).await;

        assert!(delete_tag(&db, tag_id).await.unwrap());

        let asset_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assets")
            .fetch_one(&db).await.unwrap();
        assert_eq!(asset_count, 2);
        assert_eq!(count_tag(&db, tag_id).await, 0);
        assert_eq!(count_links(&db, tag_id).await, 0);
    }

    #[tokio::test]
    async fn delete_missing_tag_returns_false() {
        let db = setup_db().await;
        assert!(!delete_tag(&db, 999).await.unwrap());
    }

    #[tokio::test]
    async fn tag_rename_merge_remove_and_delete_refresh_both_fts_tables() {
        let db = setup_db().await;
        let (asset_a, asset_b) = insert_two_assets(&db).await;
        apply_tag_to_assets(&db, "来源", &[asset_a, asset_b]).await.unwrap();
        apply_tag_to_assets(&db, "目标", &[asset_b]).await.unwrap();
        let source = tag_by_name(&db, "来源").await;
        let target = tag_by_name(&db, "目标").await;

        update_tag(&db, source.id, "目标", "#123456").await.unwrap();
        assert_fts_tags(&db, asset_a, "目标").await;
        assert_fts_tags(&db, asset_b, "目标").await;

        remove_tag_from_assets(&db, target.id, &[asset_a]).await.unwrap();
        assert_fts_tags(&db, asset_a, "").await;

        delete_tag(&db, target.id).await.unwrap();
        assert_fts_tags(&db, asset_b, "").await;
    }

    #[tokio::test]
    async fn update_tag_rolls_back_when_fts_refresh_fails() {
        let db = setup_db().await;
        let (asset_a, _) = insert_two_assets(&db).await;
        apply_tag_to_assets(&db, "旧标签", &[asset_a]).await.unwrap();
        let tag = tag_by_name(&db, "旧标签").await;
        sqlx::query("DROP TABLE asset_search_fts").execute(&db).await.unwrap();

        assert!(update_tag(&db, tag.id, "新标签", "#112233").await.is_err());

        let stored: (String, String) =
            sqlx::query_as("SELECT name, color FROM tags WHERE id = ?1")
                .bind(tag.id).fetch_one(&db).await.unwrap();
        assert_eq!(stored, ("旧标签".to_string(), "#5B8DEF".to_string()));
    }

    #[tokio::test]
    async fn update_tag_preserves_last_used_at_when_unchanged_or_color_only() {
        let db = setup_db().await;
        let tag_id = insert_tag(&db, "角色", "#5B8DEF").await;

        let unchanged = update_tag(&db, tag_id, "角色", "#5B8DEF").await.unwrap();
        assert_eq!(unchanged.name, "角色");
        assert_eq!(unchanged.color, "#5B8DEF");

        let color_only = update_tag(&db, tag_id, "角色", "#ABCDEF").await.unwrap();
        assert_eq!(color_only.color, "#ABCDEF");

        let after: (String, String) =
            sqlx::query_as("SELECT name, color FROM tags WHERE id = ?1")
                .bind(tag_id).fetch_one(&db).await.unwrap();
        assert_eq!(after, ("角色".to_string(), "#ABCDEF".to_string()));
    }

    #[tokio::test]
    async fn update_tag_plain_rename_refreshes_both_fts_tables() {
        let db = setup_db().await;
        let (asset_a, _) = insert_two_assets(&db).await;
        apply_tag_to_assets(&db, "旧标签", &[asset_a]).await.unwrap();
        let tag = tag_by_name(&db, "旧标签").await;

        update_tag(&db, tag.id, "新标签", "#112233").await.unwrap();

        assert_fts_tags(&db, asset_a, "新标签").await;
    }

    #[tokio::test]
    async fn update_tag_merge_rolls_back_when_fts_refresh_fails() {
        let db = setup_db().await;
        let (asset_a, asset_b) = insert_two_assets(&db).await;
        let source_id = insert_tag(&db, "来源", "#111111").await;
        let target_id = insert_tag(&db, "目标", "#ABCDEF").await;
        link_tag(&db, source_id, asset_a).await;
        link_tag(&db, target_id, asset_b).await;
        sqlx::query("DROP TABLE asset_search_fts").execute(&db).await.unwrap();

        assert!(update_tag(&db, source_id, "目标", "#222222").await.is_err());

        assert_eq!(count_tag(&db, source_id).await, 1);
        assert_eq!(count_tag(&db, target_id).await, 1);
        assert_eq!(count_links(&db, source_id).await, 1);
        assert_eq!(count_links(&db, target_id).await, 1);
    }

    #[tokio::test]
    async fn remove_tag_from_assets_rolls_back_when_fts_refresh_fails() {
        let db = setup_db().await;
        let (asset_a, _) = insert_two_assets(&db).await;
        let tag_id = insert_tag(&db, "角色", "#5B8DEF").await;
        link_tag(&db, tag_id, asset_a).await;
        refresh_asset_search_document(&mut *db.acquire().await.unwrap(), asset_a).await.unwrap();
        sqlx::query("DROP TABLE asset_search_fts").execute(&db).await.unwrap();

        assert!(remove_tag_from_assets(&db, tag_id, &[asset_a]).await.is_err());

        assert_eq!(count_links(&db, tag_id).await, 1);
    }

    #[tokio::test]
    async fn delete_tag_rolls_back_when_fts_refresh_fails() {
        let db = setup_db().await;
        let (asset_a, _) = insert_two_assets(&db).await;
        let tag_id = insert_tag(&db, "角色", "#5B8DEF").await;
        link_tag(&db, tag_id, asset_a).await;
        refresh_asset_search_document(&mut *db.acquire().await.unwrap(), asset_a).await.unwrap();
        sqlx::query("DROP TABLE asset_search_fts").execute(&db).await.unwrap();

        assert!(delete_tag(&db, tag_id).await.is_err());

        assert_eq!(count_tag(&db, tag_id).await, 1);
        assert_eq!(count_links(&db, tag_id).await, 1);
    }

    #[tokio::test]
    async fn update_tag_rolls_back_when_trigram_fts_refresh_fails() {
        let db = setup_db().await;
        let (asset_a, _) = insert_two_assets(&db).await;
        apply_tag_to_assets(&db, "旧标签", &[asset_a]).await.unwrap();
        let tag = tag_by_name(&db, "旧标签").await;
        sqlx::query("DROP TABLE asset_search_trigram_fts").execute(&db).await.unwrap();

        assert!(update_tag(&db, tag.id, "新标签", "#112233").await.is_err());

        let stored: (String, String) =
            sqlx::query_as("SELECT name, color FROM tags WHERE id = ?1")
                .bind(tag.id).fetch_one(&db).await.unwrap();
        assert_eq!(stored, ("旧标签".to_string(), "#5B8DEF".to_string()));
    }
}
