use crate::{db, file_actions, indexer, thumbnails, scan_service, ThumbnailDir};
use crate::scan_service::ScanRuntime;
use crate::indexer::ScannedAsset;
use serde::Serialize;
use sqlx::SqlitePool;
use std::path::Path;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<anyhow::Error> for CommandError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<sqlx::Error> for CommandError {
    fn from(value: sqlx::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

#[tauri::command]
pub async fn list_assets(db: State<'_, SqlitePool>) -> Result<Vec<crate::models::Asset>, CommandError> {
    db::list_assets(&*db, 2000, 0).await.map_err(Into::into)
}

#[tauri::command]
pub async fn list_library_folders(
    db: State<'_, SqlitePool>,
) -> Result<Vec<crate::models::LibraryFolder>, CommandError> {
    db::list_library_folders(&*db).await.map_err(Into::into)
}

#[tauri::command]
pub async fn set_asset_favorite(
    db: State<'_, SqlitePool>,
    asset_id: i64,
    is_favorite: bool,
) -> Result<(), CommandError> {
    db::set_asset_favorite(&*db, asset_id, is_favorite).await.map_err(Into::into)
}

#[tauri::command]
pub async fn apply_tag_to_assets(
    db: State<'_, SqlitePool>,
    tag_name: String,
    asset_ids: Vec<i64>,
) -> Result<(), CommandError> {
    db::apply_tag_to_assets(&*db, &tag_name, &asset_ids).await.map_err(Into::into)
}

#[tauri::command]
pub async fn open_asset_file(path: String) -> Result<(), CommandError> {
    file_actions::open_file(&path).map_err(Into::into)
}

#[tauri::command]
pub async fn reveal_asset_in_folder(path: String) -> Result<(), CommandError> {
    file_actions::reveal_in_folder(&path).map_err(Into::into)
}

#[tauri::command]
pub async fn add_library_folder(
    db: State<'_, SqlitePool>,
    name: String,
    path: String,
) -> Result<crate::models::LibraryFolder, CommandError> {
    db::create_library_folder(&*db, &name, &path).await.map_err(Into::into)
}

#[tauri::command]
pub async fn scan_library_folder(
    db: State<'_, SqlitePool>,
    thumbnail_dir: State<'_, ThumbnailDir>,
    folder_id: i64,
) -> Result<ScanResult, CommandError> {
    let folder = db::get_folder_by_id(&*db, folder_id)
        .await
        .map_err(CommandError::from)?
        .ok_or_else(|| CommandError::from(anyhow::anyhow!("folder not found")))?;

    let output = indexer::scan_folder(Path::new(&folder.path)).map_err(CommandError::from)?;
    let cache_dir = &thumbnail_dir.0;
    let found = output.assets.len();

    let mut scanned: Vec<(ScannedAsset, Option<String>, Option<i64>, Option<i64>)> = Vec::with_capacity(found);
    let mut existing_paths: Vec<String> = Vec::with_capacity(found);
    let mut skipped = output.skipped_count;

    for asset in output.assets {
        existing_paths.push(asset.absolute_path.clone());
        let (thumb_path, width, height) = if asset.asset_type == "image" {
            match thumbnails::generate_image_thumbnail(
                Path::new(&asset.absolute_path),
                &asset.absolute_path,
                &asset.modified_at,
                cache_dir,
            ) {
                Ok(result) => (Some(result.thumbnail_path), Some(result.width), Some(result.height)),
                Err(e) => {
                    eprintln!("thumbnail failed for {}: {}", asset.absolute_path, e);
                    skipped += 1;
                    (None, None, None)
                }
            }
        } else {
            (None, None, None)
        };
        scanned.push((asset, thumb_path, width, height));
    }

    let mut tx = db.begin().await.map_err(CommandError::from)?;
    let (added, updated, missing) = db::write_scan_results(&mut tx, folder_id, &scanned, &existing_paths)
        .await
        .map_err(CommandError::from)?;
    tx.commit().await.map_err(CommandError::from)?;

    Ok(ScanResult {
        found,
        added,
        updated,
        skipped,
        missing: missing as usize,
    })
}

#[tauri::command]
pub async fn list_asset_tags(
    db: State<'_, SqlitePool>,
) -> Result<Vec<(i64, String)>, CommandError> {
    db::list_asset_tags_map(&*db).await.map_err(Into::into)
}

#[tauri::command]
pub async fn start_scan(
    db: State<'_, SqlitePool>,
    thumbnail_dir: State<'_, ThumbnailDir>,
    runtime: State<'_, ScanRuntime>,
    folder_id: i64,
) -> Result<crate::models::ScanJob, CommandError> {
    let job = db::create_scan_job(&*db, folder_id).await.map_err(CommandError::from)?;
    let pool = db.inner().clone();
    let runtime_clone = runtime.inner().clone();
    let thumb_dir = thumbnail_dir.0.clone();
    let job_id = job.id;

    tauri::async_runtime::spawn(async move {
        if let Err(e) = scan_service::run_scan_job(pool.clone(), runtime_clone, thumb_dir, folder_id, job_id).await {
            let _ = db::fail_scan_job(&pool, job_id, &e.to_string()).await;
        }
    });

    Ok(job)
}

#[tauri::command]
pub async fn cancel_scan(
    db: State<'_, SqlitePool>,
    runtime: State<'_, ScanRuntime>,
    job_id: i64,
) -> Result<(), CommandError> {
    runtime.cancel(job_id);
    db::cancel_scan_job(&*db, job_id).await.map_err(CommandError::from)?;
    Ok(())
}

#[tauri::command]
pub async fn latest_scan_job(
    db: State<'_, SqlitePool>,
    folder_id: i64,
) -> Result<Option<crate::models::ScanJob>, CommandError> {
    db::latest_scan_job_for_folder(&*db, folder_id).await.map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_scan_settings(
    db: State<'_, SqlitePool>,
) -> Result<crate::models::ScanSettings, CommandError> {
    db::get_scan_settings(&*db).await.map_err(CommandError::from)
}

#[tauri::command]
pub async fn save_scan_settings(
    db: State<'_, SqlitePool>,
    settings: crate::models::ScanSettings,
) -> Result<(), CommandError> {
    db::save_scan_settings(&*db, &settings).await.map_err(CommandError::from)?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanResult {
    pub found: usize,
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
    pub missing: usize,
}
