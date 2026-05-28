use anyhow::Context;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

pub type Db = SqlitePool;

pub async fn connect(database_path: &Path) -> anyhow::Result<Db> {
    if let Some(parent) = database_path.parent() {
        std::fs::create_dir_all(parent).context("failed to create database directory")?;
    }

    let url = format!("sqlite://{}", database_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
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
    let rows = sqlx::query_as::<_, (
        i64, i64, String, String, String, String, i64, String,
        Option<i64>, Option<i64>, Option<String>, String, i64, i64, String, String
    )>(
        "SELECT id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size,
                modified_at, width, height, thumbnail_path, note, is_favorite, is_missing,
                created_at, updated_at
         FROM assets
         ORDER BY file_name
         LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Asset {
            id: row.0,
            library_folder_id: row.1,
            absolute_path: row.2,
            file_name: row.3,
            extension: row.4,
            asset_type: row.5,
            file_size: row.6,
            modified_at: row.7,
            width: row.8,
            height: row.9,
            thumbnail_path: row.10,
            note: row.11,
            is_favorite: row.12 == 1,
            is_missing: row.13 == 1,
            created_at: row.14,
            updated_at: row.15,
        })
        .collect())
}

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
