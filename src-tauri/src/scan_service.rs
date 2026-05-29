use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use sqlx::SqlitePool;
use chrono::Utc;
use walkdir::WalkDir;
use crate::db;
use crate::indexer::{classify_asset, normalize_path, should_ignore_dir, asset_type_allowed, should_generate_thumbnail, ScannedAsset};
use crate::models::AssetType;

#[derive(Clone)]
pub struct ScanRuntime {
    cancelled: Arc<tokio::sync::Mutex<HashSet<i64>>>,
    active: Arc<tokio::sync::Mutex<HashSet<i64>>>,
    starting: Arc<tokio::sync::Mutex<HashMap<i64, i64>>>,
}

impl Default for ScanRuntime {
    fn default() -> Self {
        Self {
            cancelled: Arc::new(tokio::sync::Mutex::new(HashSet::new())),
            active: Arc::new(tokio::sync::Mutex::new(HashSet::new())),
            starting: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

pub struct StartingGuard {
    starting: Arc<tokio::sync::Mutex<HashMap<i64, i64>>>,
    folder_id: i64,
}

impl Drop for StartingGuard {
    fn drop(&mut self) {
        if let Ok(mut map) = self.starting.try_lock() {
            map.remove(&self.folder_id);
        } else {
            let starting = self.starting.clone();
            let folder_id = self.folder_id;
            tokio::spawn(async move {
                starting.lock().await.remove(&folder_id);
            });
        }
    }
}

impl ScanRuntime {
    pub async fn start(&self, job_id: i64) {
        self.active.lock().await.insert(job_id);
    }

    pub async fn finish(&self, job_id: i64) {
        self.active.lock().await.remove(&job_id);
        self.clear(job_id).await;
    }

    pub async fn is_active(&self, job_id: i64) -> bool {
        self.active.lock().await.contains(&job_id)
    }

    pub async fn cancel(&self, job_id: i64) {
        self.cancelled.lock().await.insert(job_id);
    }

    pub async fn is_cancelled(&self, job_id: i64) -> bool {
        self.cancelled.lock().await.contains(&job_id)
    }

    pub async fn clear(&self, job_id: i64) {
        self.cancelled.lock().await.remove(&job_id);
    }

    pub async fn guard_starting(&self, folder_id: i64, job_id: i64) -> StartingGuard {
        self.starting.lock().await.insert(folder_id, job_id);
        StartingGuard {
            starting: self.starting.clone(),
            folder_id,
        }
    }

    pub async fn is_starting_job(&self, folder_id: i64, job_id: i64) -> bool {
        self.starting.lock().await.get(&folder_id) == Some(&job_id)
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
            "SELECT absolute_path, file_size, modified_at, is_missing FROM assets WHERE absolute_path IN ({})",
            placeholders.join(",")
        );
        let mut query = sqlx::query_as::<_, (String, i64, String, i64)>(&sql);
        for asset in batch {
            query = query.bind(&asset.absolute_path);
        }
        let existing_rows = query.fetch_all(&mut *tx).await?;
        let existing: std::collections::HashMap<String, (i64, String, bool)> = existing_rows
            .into_iter()
            .map(|r| (r.0, (r.1, r.2, r.3 == 1)))
            .collect();

        let now = Utc::now().to_rfc3339();

        for asset in batch {
            if let Some((size, modified, was_missing)) = existing.get(&asset.absolute_path) {
                if *size == asset.file_size && *modified == asset.modified_at {
                    counters.unchanged += 1;
                    if *was_missing {
                        sqlx::query(
                            "UPDATE assets SET is_missing = 0, updated_at = ?1 WHERE absolute_path = ?2"
                        )
                        .bind(&now)
                        .bind(&asset.absolute_path)
                        .execute(&mut *tx)
                        .await?;
                    }
                    continue;
                }
            }

            let was_existing = existing.contains_key(&asset.absolute_path);

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
                    width = COALESCE(excluded.width, assets.width),
                    height = COALESCE(excluded.height, assets.height),
                    thumbnail_path = COALESCE(excluded.thumbnail_path, assets.thumbnail_path),
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
            .bind(asset.width)
            .bind(asset.height)
            .bind(asset.thumbnail_path.as_deref())
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

async fn flush_batch(
    db_pool: &SqlitePool,
    folder_id: i64,
    job_id: i64,
    batch: &mut Vec<ScannedAsset>,
    counters: &mut ScanCounters,
    thumbnail_dir: &std::path::Path,
    settings: &crate::models::ScanSettings,
) -> anyhow::Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut handles = Vec::new();
    for (idx, asset) in batch.iter().enumerate() {
        if should_generate_thumbnail(&asset.extension, settings) {
            let abs_path = asset.absolute_path.clone();
            let modified = asset.modified_at.clone();
            let thumb_dir = thumbnail_dir.to_path_buf();
            let handle = tokio::spawn(async move {
                let result = crate::thumbnails::generate_image_thumbnail_async(
                    std::path::Path::new(&abs_path),
                    &abs_path,
                    &modified,
                    &thumb_dir,
                ).await;
                (idx, result)
            });
            handles.push(handle);
        }
    }

    for handle in handles {
        let (idx, result) = handle.await.map_err(|e| anyhow::anyhow!("thumbnail task failed: {}", e))?;
        match result {
            Ok(r) => {
                batch[idx].thumbnail_path = Some(r.thumbnail_path);
                batch[idx].width = Some(r.width);
                batch[idx].height = Some(r.height);
                batch[idx].thumbnail_status = "ready".to_string();
            }
            Err(e) => {
                batch[idx].thumbnail_status = "failed".to_string();
                batch[idx].thumbnail_error = Some(e.to_string());
            }
        }
    }

    persist_batch(db_pool, folder_id, job_id, batch, counters).await?;
    db::update_scan_job_progress(
        db_pool,
        job_id,
        counters.found as i64,
        counters.added as i64,
        counters.updated as i64,
        counters.unchanged as i64,
        counters.skipped as i64,
        batch.last().map(|a| a.absolute_path.as_str()),
    ).await?;
    batch.clear();
    Ok(())
}

pub async fn run_scan_job(
    db_pool: SqlitePool,
    runtime: ScanRuntime,
    thumbnail_dir: std::path::PathBuf,
    folder_id: i64,
    job_id: i64,
) -> anyhow::Result<()> {
    runtime.start(job_id).await;
    let result = async {
        let folder = db::get_folder_by_id(&db_pool, folder_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("folder not found"))?;

        let settings = db::get_scan_settings(&db_pool).await?;

        let mut counters = ScanCounters::default();
        let mut batch: Vec<ScannedAsset> = Vec::with_capacity(SCAN_BATCH_SIZE);

        let mut it = WalkDir::new(&folder.path).follow_links(false).into_iter();
        while let Some(entry) = it.next() {
            if runtime.is_cancelled(job_id).await {
                flush_batch(&db_pool, folder_id, job_id, &mut batch, &mut counters, &thumbnail_dir, &settings).await?;
                runtime.clear(job_id).await;
                db::cancel_scan_job(&db_pool, job_id).await?;
                return Ok(());
            }

            let entry = match entry {
                Ok(value) => value,
                Err(_) => {
                    counters.skipped += 1;
                    continue;
                }
            };

            if entry.file_type().is_dir() {
                if should_ignore_dir(entry.path(), &settings) {
                    it.skip_current_dir();
                }
                continue;
            }

            if !entry.file_type().is_file() {
                continue;
            }

            let metadata = match std::fs::metadata(entry.path()) {
                Ok(value) => value,
                Err(_) => {
                    counters.skipped += 1;
                    continue;
                }
            };

            let asset_type = classify_asset(entry.path());
            let extension = entry
                .path()
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();

            if asset_type == AssetType::Other || !asset_type_allowed(asset_type.as_str(), &extension, &settings) {
                counters.skipped += 1;
                continue;
            }

            let modified_at: chrono::DateTime<Utc> = metadata.modified()?.into();
            let absolute_path = normalize_path(entry.path())?;
            let file_name = entry
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();

            let scanned = ScannedAsset {
                absolute_path,
                file_name,
                extension: extension.clone(),
                asset_type: asset_type.as_str().to_string(),
                file_size: metadata.len() as i64,
                modified_at: modified_at.to_rfc3339(),
                ..Default::default()
            };

            batch.push(scanned);

            if batch.len() >= SCAN_BATCH_SIZE {
                flush_batch(&db_pool, folder_id, job_id, &mut batch, &mut counters, &thumbnail_dir, &settings).await?;
            }
        }

        flush_batch(&db_pool, folder_id, job_id, &mut batch, &mut counters, &thumbnail_dir, &settings).await?;

        if runtime.is_cancelled(job_id).await {
            runtime.clear(job_id).await;
            db::cancel_scan_job(&db_pool, job_id).await?;
            return Ok(());
        }

        let missing = db::mark_missing_assets_from_seen(&db_pool, folder_id, job_id).await?;

        if runtime.is_cancelled(job_id).await {
            runtime.clear(job_id).await;
            db::cancel_scan_job(&db_pool, job_id).await?;
            return Ok(());
        }

        let rows = db::finish_scan_job(&db_pool, job_id, missing as i64).await?;
        if rows > 0 {
            db::update_folder_last_scanned(&db_pool, folder_id).await?;
        }

        Ok(())
    }.await;
    runtime.finish(job_id).await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    async fn setup_test_db() -> (SqlitePool, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = crate::db::connect(&db_path).await.unwrap();
        (pool, temp_dir)
    }

    async fn create_test_folder(db: &SqlitePool, base: &tempfile::TempDir, name: &str) -> crate::models::LibraryFolder {
        let path = base.path().join(name);
        std::fs::create_dir_all(&path).unwrap();
        crate::db::create_library_folder(db, name, path.to_str().unwrap()).await.unwrap()
    }

    fn write_file(dir: &std::path::Path, name: &str, content: &[u8]) {
        let path = dir.join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(content).unwrap();
    }

    #[tokio::test]
    async fn scan_completes_and_updates_last_scanned_at() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        let asset_dir = std::path::Path::new(&folder.path);
        write_file(asset_dir, "icon.png", b"fake");

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        run_scan_job(db.clone(), ScanRuntime::default(), tmp.path().join("thumbs"), folder.id, job.id).await.unwrap();

        let folder = crate::db::get_folder_by_id(&db, folder.id).await.unwrap().unwrap();
        assert!(folder.last_scanned_at.is_some());

        let assets = crate::db::list_assets(&db, 1000, 0).await.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].file_name, "icon.png");

        let job = crate::db::latest_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(job.status, crate::models::ScanJobStatus::Completed);
        assert_eq!(job.found_count, 1);
        assert_eq!(job.added_count, 1);
    }

    #[tokio::test]
    async fn cancelled_scan_keeps_inserted_assets() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        let asset_dir = std::path::Path::new(&folder.path);
        for i in 0..1000 {
            write_file(asset_dir, &format!("img_{}.png", i), b"fake");
        }

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        let runtime = ScanRuntime::default();
        let runtime_clone = runtime.clone();
        let db_clone = db.clone();
        let job_id = job.id;

        let handle = tokio::spawn(async move {
            run_scan_job(db_clone, runtime_clone, tmp.path().join("thumbs"), folder.id, job_id).await
        });

        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        runtime.cancel(job_id).await;

        let result = handle.await.unwrap();
        assert!(result.is_ok());

        let job = crate::db::latest_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(job.status, crate::models::ScanJobStatus::Cancelled);

        let assets = crate::db::list_assets(&db, 10000, 0).await.unwrap();
        assert!(!assets.is_empty(), "some assets should remain after cancellation");
    }

    #[tokio::test]
    async fn reappeared_unchanged_asset_clears_is_missing() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        let asset_dir = std::path::Path::new(&folder.path);
        write_file(asset_dir, "icon.png", b"same");

        let job1 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        run_scan_job(db.clone(), ScanRuntime::default(), tmp.path().join("thumbs"), folder.id, job1.id).await.unwrap();

        let asset = crate::db::list_assets(&db, 1000, 0).await.unwrap().into_iter().next().unwrap();
        sqlx::query("UPDATE assets SET is_missing = 1 WHERE id = ?1")
            .bind(asset.id)
            .execute(&db).await.unwrap();

        let job2 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        run_scan_job(db.clone(), ScanRuntime::default(), tmp.path().join("thumbs"), folder.id, job2.id).await.unwrap();

        let asset = crate::db::list_assets(&db, 1000, 0).await.unwrap().into_iter().next().unwrap();
        assert!(!asset.is_missing);
    }

    #[tokio::test]
    async fn upsert_preserves_existing_thumbnail() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        let asset_dir = std::path::Path::new(&folder.path);
        write_file(asset_dir, "icon.png", b"v1");

        let job1 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        run_scan_job(db.clone(), ScanRuntime::default(), tmp.path().join("thumbs"), folder.id, job1.id).await.unwrap();

        let asset = crate::db::list_assets(&db, 1000, 0).await.unwrap().into_iter().next().unwrap();
        sqlx::query("UPDATE assets SET thumbnail_path = 'thumbs/icon.png' WHERE id = ?1")
            .bind(asset.id)
            .execute(&db).await.unwrap();

        write_file(asset_dir, "icon.png", b"v2-different");

        let job2 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        run_scan_job(db.clone(), ScanRuntime::default(), tmp.path().join("thumbs"), folder.id, job2.id).await.unwrap();

        let asset = crate::db::list_assets(&db, 1000, 0).await.unwrap().into_iter().next().unwrap();
        assert_eq!(asset.thumbnail_path, Some("thumbs/icon.png".to_string()));
    }

    #[tokio::test]
    async fn second_scan_reports_unchanged_count() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        let asset_dir = std::path::Path::new(&folder.path);
        write_file(asset_dir, "a.png", b"1");
        write_file(asset_dir, "b.png", b"2");

        let job1 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        run_scan_job(db.clone(), ScanRuntime::default(), tmp.path().join("thumbs"), folder.id, job1.id).await.unwrap();
        let job1 = crate::db::latest_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(job1.unchanged_count, 0);

        let job2 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        run_scan_job(db.clone(), ScanRuntime::default(), tmp.path().join("thumbs"), folder.id, job2.id).await.unwrap();
        let job2 = crate::db::latest_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(job2.unchanged_count, 2);
    }

    #[tokio::test]
    async fn latest_scan_job_returns_running_for_active_job() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        let latest = crate::db::latest_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(latest.id, job.id);
        assert_eq!(latest.status, crate::models::ScanJobStatus::Running);
    }

    #[tokio::test]
    async fn empty_directory_scan_cancels_at_end_of_scan() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        let runtime = ScanRuntime::default();
        runtime.cancel(job.id).await;

        run_scan_job(db.clone(), runtime, tmp.path().join("thumbs"), folder.id, job.id).await.unwrap();

        let job = crate::db::latest_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(job.status, crate::models::ScanJobStatus::Cancelled);
    }

    #[tokio::test]
    async fn finish_scan_job_only_updates_running_job() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        write_file(std::path::Path::new(&folder.path), "icon.png", b"fake");

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        run_scan_job(db.clone(), ScanRuntime::default(), tmp.path().join("thumbs"), folder.id, job.id).await.unwrap();

        let rows = crate::db::finish_scan_job(&db, job.id, 0).await.unwrap();
        assert_eq!(rows, 0, "finish_scan_job should not affect already-completed job");
    }

    #[tokio::test]
    async fn cancel_scan_job_only_updates_running_job() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        crate::db::cancel_scan_job(&db, job.id).await.unwrap();

        let rows = crate::db::cancel_scan_job(&db, job.id).await.unwrap();
        assert_eq!(rows, 0, "cancel_scan_job should not affect already-cancelled job");
    }

    #[tokio::test]
    async fn duplicate_running_scan_job_is_rejected() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;

        let _job1 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        let result = crate::db::create_scan_job(&db, folder.id).await;
        assert!(result.is_err(), "second running scan job for same folder should be rejected");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("UNIQUE constraint failed") || err_msg.to_lowercase().contains("unique"),
            "expected unique constraint error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn get_scan_job_maps_all_fields() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        let fetched = crate::db::get_scan_job(&db, job.id).await.unwrap().unwrap();

        assert_eq!(fetched.id, job.id);
        assert_eq!(fetched.library_folder_id, folder.id);
        assert_eq!(fetched.status, crate::models::ScanJobStatus::Running);
        assert!(!fetched.started_at.is_empty());
    }

    #[tokio::test]
    async fn migration_deduplicates_duplicate_running_jobs() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;

        // Drop the partial unique index so we can insert duplicates manually
        sqlx::query("DROP INDEX IF EXISTS idx_scan_jobs_folder_running")
            .execute(&db).await.unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        for _ in 0..3 {
            sqlx::query("INSERT INTO scan_jobs (library_folder_id, status, started_at) VALUES (?1, 'running', ?2)")
                .bind(folder.id)
                .bind(&now)
                .execute(&db).await.unwrap();
        }

        // Re-run the migration SQL
        sqlx::query(
            "UPDATE scan_jobs
             SET status = 'failed',
                 finished_at = datetime('now'),
                 current_path = NULL,
                 error_message = 'interrupted by migration: duplicate running job deduplication'
             WHERE id NOT IN (
                 SELECT MAX(id)
                 FROM scan_jobs
                 WHERE status = 'running'
                 GROUP BY library_folder_id
             )
               AND status = 'running'"
        ).execute(&db).await.unwrap();

        sqlx::query(
            "CREATE UNIQUE INDEX idx_scan_jobs_folder_running ON scan_jobs(library_folder_id) WHERE status = 'running'"
        ).execute(&db).await.unwrap();

        let running_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM scan_jobs WHERE library_folder_id = ?1 AND status = 'running'"
        ).bind(folder.id).fetch_one(&db).await.unwrap();
        assert_eq!(running_count, 1);

        let failed_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM scan_jobs WHERE library_folder_id = ?1 AND status = 'failed'"
        ).bind(folder.id).fetch_one(&db).await.unwrap();
        assert_eq!(failed_count, 2);
    }

    #[tokio::test]
    async fn cancelled_scan_updates_progress_counts() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        let asset_dir = std::path::Path::new(&folder.path);
        for i in 0..100 {
            write_file(asset_dir, &format!("img_{}.png", i), b"fake");
        }

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        let runtime = ScanRuntime::default();
        let runtime_clone = runtime.clone();
        let db_clone = db.clone();
        let job_id = job.id;

        let handle = tokio::spawn(async move {
            run_scan_job(db_clone, runtime_clone, tmp.path().join("thumbs"), folder.id, job_id).await
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        runtime.cancel(job_id).await;

        let result = handle.await.unwrap();
        assert!(result.is_ok());

        let job = crate::db::latest_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(job.status, crate::models::ScanJobStatus::Cancelled);
        assert!(job.found_count > 0, "found_count should reflect flushed assets, got {}", job.found_count);
    }

    #[tokio::test]
    async fn stale_running_job_can_be_recovered_and_replaced() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;

        // Create a stale running job (no active worker)
        let stale_job = crate::db::create_scan_job(&db, folder.id).await.unwrap();

        // Direct insert of a second running job should fail due to unique index
        let result = crate::db::create_scan_job(&db, folder.id).await;
        assert!(result.is_err());

        // Recover stale job by marking it failed
        let rows = crate::db::fail_scan_job(&db, stale_job.id, "stale job recovery test").await.unwrap();
        assert_eq!(rows, 1);

        // Now we can create a new running job
        let new_job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        assert_ne!(new_job.id, stale_job.id);
        assert_eq!(new_job.status, crate::models::ScanJobStatus::Running);
    }

    #[tokio::test]
    async fn cancel_scan_updates_db_for_stale_job() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        // Simulate cancel_scan command behavior (cancel runtime + update DB)
        let _ = crate::db::cancel_scan_job(&db, job.id).await.unwrap();

        let job = crate::db::get_scan_job(&db, job.id).await.unwrap().unwrap();
        assert_eq!(job.status, crate::models::ScanJobStatus::Cancelled);
    }

    #[tokio::test]
    async fn running_scan_job_for_folder_returns_only_running() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        write_file(std::path::Path::new(&folder.path), "icon.png", b"fake");

        let job1 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        run_scan_job(db.clone(), ScanRuntime::default(), tmp.path().join("thumbs"), folder.id, job1.id).await.unwrap();

        // latest_scan_job_for_folder returns the completed job
        let latest = crate::db::latest_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(latest.status, crate::models::ScanJobStatus::Completed);

        // running_scan_job_for_folder returns None because nothing is running
        let running = crate::db::running_scan_job_for_folder(&db, folder.id).await.unwrap();
        assert!(running.is_none());
    }

    #[tokio::test]
    async fn runtime_start_prevents_stale_misclassification() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;

        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        let runtime = ScanRuntime::default();

        // Simulate what start_scan does: register active BEFORE spawning worker
        runtime.start(job.id).await;
        assert!(runtime.is_active(job.id).await, "job should be active immediately after start");

        // A concurrent check would see is_active=true and return the existing job
        let running = crate::db::running_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(running.id, job.id);

        // Simulate worker completion
        runtime.finish(job.id).await;
        assert!(!runtime.is_active(job.id).await, "job should not be active after finish");
    }

    #[tokio::test]
    async fn stale_running_with_newer_completed_job_recovery() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        write_file(std::path::Path::new(&folder.path), "icon.png", b"fake");

        // Create an old stale running job
        let stale_job = crate::db::create_scan_job(&db, folder.id).await.unwrap();

        // Manually insert a newer completed job (simulating the regression scenario)
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO scan_jobs (library_folder_id, status, started_at, finished_at)
             VALUES (?1, 'completed', ?2, ?2)"
        )
        .bind(folder.id)
        .bind(&now)
        .execute(&db)
        .await
        .unwrap();

        // latest_scan_job_for_folder returns the newer completed job
        let latest = crate::db::latest_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(latest.status, crate::models::ScanJobStatus::Completed);
        assert_ne!(latest.id, stale_job.id);

        // running_scan_job_for_folder returns the old stale running job
        let running = crate::db::running_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(running.id, stale_job.id);
        assert_eq!(running.status, crate::models::ScanJobStatus::Running);

        // Simulate stale recovery (what start_scan does)
        let runtime = ScanRuntime::default();
        assert!(!runtime.is_active(stale_job.id).await, "stale job should not be active");
        let rows = crate::db::fail_scan_job(&db, stale_job.id, "stale recovery test").await.unwrap();
        assert_eq!(rows, 1);

        // Now we can create a new running job
        let new_job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        assert_eq!(new_job.status, crate::models::ScanJobStatus::Running);
    }

    #[tokio::test]
    async fn start_scan_retry_simulation_hits_unique_and_returns_active() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        write_file(std::path::Path::new(&folder.path), "icon.png", b"fake");

        // Simulate first start_scan: create job and register active
        let job1 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        let runtime = ScanRuntime::default();
        runtime.start(job1.id).await;

        // Simulate concurrent second start_scan: unique constraint hits
        let result = crate::db::create_scan_job(&db, folder.id).await;
        assert!(result.is_err(), "unique index should block second running job");

        // Simulate start_scan retry logic: find running job, check is_active
        let running = crate::db::running_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(running.id, job1.id);
        assert!(runtime.is_active(running.id).await, "active job should be returned instead of creating new one");

        // After worker finishes, a new start_scan should recover stale and create new job
        runtime.finish(job1.id).await;
        let rows = crate::db::fail_scan_job(&db, job1.id, "stale recovery").await.unwrap();
        assert_eq!(rows, 1);

        let job2 = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        assert_ne!(job2.id, job1.id);
        assert_eq!(job2.status, crate::models::ScanJobStatus::Running);
    }

    #[tokio::test]
    async fn starting_marker_prevents_concurrent_misclassification() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        write_file(std::path::Path::new(&folder.path), "icon.png", b"fake");

        let runtime = ScanRuntime::default();

        // Simulate Thread A: create job then mark starting (but haven't called start yet)
        let job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        let _starting_guard = runtime.guard_starting(folder.id, job.id).await;
        assert!(!runtime.is_active(job.id).await, "job should not be active yet");

        // Simulate Thread B: create fails, finds running job, checks is_active=false
        let result = crate::db::create_scan_job(&db, folder.id).await;
        assert!(result.is_err(), "unique index should block second running job");

        let running = crate::db::running_scan_job_for_folder(&db, folder.id).await.unwrap().unwrap();
        assert_eq!(running.id, job.id);
        assert!(!runtime.is_active(running.id).await, "job is not active yet");

        // Without the starting marker, this would be misclassified as stale and failed.
        // With the job-specific marker, is_starting_job returns true, so the concurrent caller
        // should treat it as a legitimate in-progress creation.
        assert!(runtime.is_starting_job(folder.id, job.id).await, "job should be marked as starting for this folder");

        // A stale job for the same folder would NOT be protected
        let fake_job_id = 99999;
        assert!(!runtime.is_starting_job(folder.id, fake_job_id).await, "different job id should not be protected");

        // Thread A now finishes registration
        runtime.start(job.id).await;
        assert!(runtime.is_active(job.id).await);

        // Thread B would return the existing job instead of failing it
        runtime.finish(job.id).await;
        let rows = crate::db::fail_scan_job(&db, job.id, "cleanup").await.unwrap();
        assert_eq!(rows, 1);

        drop(_starting_guard);
        assert!(!runtime.is_starting_job(folder.id, job.id).await);
    }

    #[tokio::test]
    async fn stale_job_without_starting_marker_is_recovered() {
        let (db, tmp) = setup_test_db().await;
        let folder = create_test_folder(&db, &tmp, "assets").await;
        write_file(std::path::Path::new(&folder.path), "icon.png", b"fake");

        let runtime = ScanRuntime::default();

        // Create a stale running job (no starting marker, not active)
        let stale_job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        assert!(!runtime.is_active(stale_job.id).await);
        assert!(!runtime.is_starting_job(folder.id, stale_job.id).await);

        // Simulate retry branch: is_active=false, is_starting_job=false
        // This should be treated as stale and failed
        let rows = crate::db::fail_scan_job(&db, stale_job.id, "stale recovery").await.unwrap();
        assert_eq!(rows, 1);

        // Now a new scan can be created
        let new_job = crate::db::create_scan_job(&db, folder.id).await.unwrap();
        assert_ne!(new_job.id, stale_job.id);
        assert_eq!(new_job.status, crate::models::ScanJobStatus::Running);
    }
}
