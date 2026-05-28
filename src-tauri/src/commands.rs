use crate::{db, file_actions, scan_service, ThumbnailDir};
use crate::scan_service::ScanRuntime;
use serde::Serialize;
use sqlx::SqlitePool;
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
pub async fn list_asset_tags(
    db: State<'_, SqlitePool>,
) -> Result<Vec<(i64, String)>, CommandError> {
    db::list_asset_tags_map(&*db).await.map_err(Into::into)
}

fn is_unique_constraint_error(err: &anyhow::Error) -> bool {
    let sqlx_err = err.downcast_ref::<sqlx::Error>()
        .or_else(|| {
            err.root_cause()
                .downcast_ref::<sqlx::Error>()
        });
    if let Some(sqlx_err) = sqlx_err {
        if let sqlx::Error::Database(db_err) = sqlx_err {
            return db_err.message().contains("UNIQUE constraint failed");
        }
    }
    false
}

#[tauri::command]
pub async fn start_scan(
    db: State<'_, SqlitePool>,
    thumbnail_dir: State<'_, ThumbnailDir>,
    runtime: State<'_, ScanRuntime>,
    folder_id: i64,
) -> Result<crate::models::ScanJob, CommandError> {
    let result = async {
        loop {
            let job = match db::create_scan_job(&*db, folder_id).await {
                Ok(job) => job,
                Err(e) => {
                    if is_unique_constraint_error(&e) {
                        if let Some(running) = db::running_scan_job_for_folder(&*db, folder_id).await.map_err(CommandError::from)? {
                            if runtime.is_active(running.id).await || runtime.is_starting_job(folder_id, running.id).await {
                                return Ok(running);
                            }
                            let _ = db::fail_scan_job(&*db, running.id, "interrupted: stale scan job recovered").await;
                            continue;
                        }
                        return Err(CommandError::from(anyhow::anyhow!("scan job state changed concurrently")));
                    }
                    return Err(CommandError::from(e));
                }
            };

            let _starting_guard = runtime.guard_starting(folder_id, job.id).await;
            runtime.start(job.id).await;
            drop(_starting_guard);

            let pool = db.inner().clone();
            let runtime_clone = runtime.inner().clone();
            let thumb_dir = thumbnail_dir.0.clone();
            let job_id = job.id;

            tauri::async_runtime::spawn(async move {
                let result = scan_service::run_scan_job(pool.clone(), runtime_clone.clone(), thumb_dir, folder_id, job_id).await;
                if let Err(e) = result {
                    let _ = db::fail_scan_job(&pool, job_id, &e.to_string()).await;
                }
            });

            return Ok(job);
        }
    }.await;
    result
}

#[tauri::command]
pub async fn cancel_scan(
    db: State<'_, SqlitePool>,
    runtime: State<'_, ScanRuntime>,
    job_id: i64,
) -> Result<(), CommandError> {
    if runtime.is_active(job_id).await {
        runtime.cancel(job_id).await;
    } else {
        db::cancel_scan_job(&*db, job_id).await.map_err(CommandError::from)?;
    }
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
