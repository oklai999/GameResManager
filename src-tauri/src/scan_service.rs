use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use sqlx::SqlitePool;
use chrono::Utc;
use std::path::Path;
use crate::db;
use crate::indexer::{scan_folder_with_settings, ScannedAsset};

#[derive(Clone, Default)]
pub struct ScanRuntime {
    cancelled: Arc<Mutex<HashSet<i64>>>,
}

impl ScanRuntime {
    pub fn cancel(&self, job_id: i64) {
        self.cancelled.lock().unwrap().insert(job_id);
    }

    pub fn is_cancelled(&self, job_id: i64) -> bool {
        self.cancelled.lock().unwrap().contains(&job_id)
    }

    pub fn clear(&self, job_id: i64) {
        self.cancelled.lock().unwrap().remove(&job_id);
    }
}

#[derive(Debug, Default)]
pub struct ScanCounters {
    pub found: usize,
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub skipped: usize,
}

const SCAN_BATCH_SIZE: usize = 500;

async fn persist_batch(
    db_pool: &SqlitePool,
    folder_id: i64,
    job_id: i64,
    batch: &[ScannedAsset],
    counters: &mut ScanCounters,
) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    for asset in batch {
        sqlx::query(
            "INSERT OR IGNORE INTO scan_seen_paths (scan_job_id, absolute_path) VALUES (?1, ?2)"
        )
        .bind(job_id)
        .bind(&asset.absolute_path)
        .execute(&mut *tx)
        .await?;
    }

    if !batch.is_empty() {
        let placeholders: Vec<String> = batch.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT absolute_path, file_size, modified_at FROM assets WHERE absolute_path IN ({})",
            placeholders.join(",")
        );
        let mut query = sqlx::query_as::<_, (String, i64, String)>(&sql);
        for asset in batch {
            query = query.bind(&asset.absolute_path);
        }
        let existing_rows = query.fetch_all(&mut *tx).await?;
        let existing: std::collections::HashMap<String, (i64, String)> = existing_rows
            .into_iter()
            .map(|r| (r.0, (r.1, r.2)))
            .collect();

        let now = Utc::now().to_rfc3339();

        for asset in batch {
            if let Some((size, modified)) = existing.get(&asset.absolute_path) {
                if *size == asset.file_size && *modified == asset.modified_at {
                    counters.unchanged += 1;
                    continue;
                }
            }

            let was_existing = existing.contains_key(&asset.absolute_path);

            sqlx::query(
                "INSERT INTO assets (
                    library_folder_id, absolute_path, file_name, extension, asset_type, file_size,
                    modified_at, width, height, thumbnail_path, created_at, updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL, NULL, ?8, ?9)
                ON CONFLICT(absolute_path) DO UPDATE SET
                    file_name = excluded.file_name,
                    extension = excluded.extension,
                    asset_type = excluded.asset_type,
                    file_size = excluded.file_size,
                    modified_at = excluded.modified_at,
                    width = NULL,
                    height = NULL,
                    thumbnail_path = NULL,
                    is_missing = 0,
                    updated_at = excluded.updated_at"
            )
            .bind(folder_id)
            .bind(&asset.absolute_path)
            .bind(&asset.file_name)
            .bind(&asset.extension)
            .bind(&asset.asset_type)
            .bind(asset.file_size)
            .bind(&asset.modified_at)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;

            if was_existing {
                counters.updated += 1;
            } else {
                counters.added += 1;
            }
        }
    }

    tx.commit().await?;
    counters.found += batch.len();
    Ok(())
}

pub async fn run_scan_job(
    db_pool: SqlitePool,
    runtime: ScanRuntime,
    _thumbnail_dir: std::path::PathBuf,
    folder_id: i64,
    job_id: i64,
) -> anyhow::Result<()> {
    let folder = db::get_folder_by_id(&db_pool, folder_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("folder not found"))?;

    let settings = db::get_scan_settings(&db_pool).await?;
    let output = scan_folder_with_settings(Path::new(&folder.path), &settings)?;

    let mut counters = ScanCounters {
        skipped: output.skipped_count,
        ..Default::default()
    };

    let mut batch: Vec<ScannedAsset> = Vec::with_capacity(SCAN_BATCH_SIZE);

    for asset in output.assets {
        if runtime.is_cancelled(job_id) {
            runtime.clear(job_id);
            return Ok(());
        }

        batch.push(asset);

        if batch.len() >= SCAN_BATCH_SIZE {
            persist_batch(&db_pool, folder_id, job_id, &batch, &mut counters).await?;
            db::update_scan_job_progress(
                &db_pool,
                job_id,
                counters.found as i64,
                counters.added as i64,
                counters.updated as i64,
                counters.unchanged as i64,
                counters.skipped as i64,
                batch.last().map(|a| a.absolute_path.as_str()),
            ).await?;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        persist_batch(&db_pool, folder_id, job_id, &batch, &mut counters).await?;
        db::update_scan_job_progress(
            &db_pool,
            job_id,
            counters.found as i64,
            counters.added as i64,
            counters.updated as i64,
            counters.unchanged as i64,
            counters.skipped as i64,
            batch.last().map(|a| a.absolute_path.as_str()),
        ).await?;
    }

    let missing = db::mark_missing_assets_from_seen(&db_pool, folder_id, job_id).await?;
    db::finish_scan_job(&db_pool, job_id, missing as i64).await?;

    Ok(())
}
