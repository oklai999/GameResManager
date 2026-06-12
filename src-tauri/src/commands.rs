use crate::{db, file_actions, indexer, scan_service, search, ThumbnailDir};
use crate::scan_service::ScanRuntime;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

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

#[tauri::command]
pub async fn list_tags(
    db: State<'_, SqlitePool>,
) -> Result<Vec<crate::models::Tag>, CommandError> {
    db::list_tags(&*db).await.map_err(Into::into)
}

#[tauri::command]
pub async fn list_recent_tags(
    db: State<'_, SqlitePool>,
    limit: i64,
) -> Result<Vec<crate::models::Tag>, CommandError> {
    db::list_recent_tags(&*db, limit).await.map_err(Into::into)
}

#[tauri::command]
pub async fn get_asset_tags(
    db: State<'_, SqlitePool>,
    asset_id: i64,
) -> Result<Vec<String>, CommandError> {
    db::list_asset_tags(&*db, asset_id).await.map_err(Into::into)
}

#[tauri::command]
pub async fn list_common_tags(
    db: State<'_, SqlitePool>,
    asset_ids: Vec<i64>,
) -> Result<Vec<String>, CommandError> {
    db::list_common_tags(&*db, &asset_ids).await.map_err(Into::into)
}

#[tauri::command]
pub async fn update_tag(
    db: State<'_, SqlitePool>,
    tag_id: i64,
    name: String,
    color: String,
) -> Result<crate::models::Tag, CommandError> {
    db::update_tag(&*db, tag_id, &name, &color)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn remove_tag_from_assets(
    db: State<'_, SqlitePool>,
    tag_id: i64,
    asset_ids: Vec<i64>,
) -> Result<u64, CommandError> {
    db::remove_tag_from_assets(&*db, tag_id, &asset_ids)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn delete_tag(
    db: State<'_, SqlitePool>,
    tag_id: i64,
) -> Result<bool, CommandError> {
    db::delete_tag(&*db, tag_id).await.map_err(Into::into)
}

#[tauri::command]
pub async fn list_collections(
    db: State<'_, SqlitePool>,
) -> Result<Vec<crate::models::Collection>, CommandError> {
    db::list_collections(&*db).await.map_err(Into::into)
}

#[tauri::command]
pub async fn create_collection(
    db: State<'_, SqlitePool>,
    name: String,
    description: String,
) -> Result<crate::models::Collection, CommandError> {
    db::create_collection(&*db, &name, &description).await.map_err(Into::into)
}

#[tauri::command]
pub async fn add_assets_to_collection(
    db: State<'_, SqlitePool>,
    collection_id: i64,
    asset_ids: Vec<i64>,
) -> Result<(), CommandError> {
    db::add_assets_to_collection(&*db, collection_id, &asset_ids).await.map_err(Into::into)
}

#[tauri::command]
pub async fn remove_asset_from_collection(
    db: State<'_, SqlitePool>,
    collection_id: i64,
    asset_id: i64,
) -> Result<(), CommandError> {
    db::remove_asset_from_collection(&*db, collection_id, asset_id).await.map_err(Into::into)
}

#[tauri::command]
pub async fn update_collection(
    db: State<'_, SqlitePool>,
    collection_id: i64,
    name: String,
    description: String,
) -> Result<crate::models::Collection, CommandError> {
    db::update_collection(&*db, collection_id, &name, &description)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn remove_assets_from_collection(
    db: State<'_, SqlitePool>,
    collection_id: i64,
    asset_ids: Vec<i64>,
) -> Result<(), CommandError> {
    db::remove_assets_from_collection(&*db, collection_id, &asset_ids)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn delete_collection(
    db: State<'_, SqlitePool>,
    collection_id: i64,
) -> Result<bool, CommandError> {
    db::delete_collection(&*db, collection_id)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn list_collection_assets(
    db: State<'_, SqlitePool>,
    collection_id: i64,
) -> Result<Vec<i64>, CommandError> {
    db::list_collection_assets(&*db, collection_id).await.map_err(Into::into)
}

#[tauri::command]
pub async fn update_asset_note(
    db: State<'_, SqlitePool>,
    asset_id: i64,
    note: String,
) -> Result<crate::models::Asset, CommandError> {
    db::update_asset_note(&*db, asset_id, &note).await.map_err(Into::into)
}

#[tauri::command]
pub async fn asset_thumbnail_url(
    db: State<'_, SqlitePool>,
    asset_id: i64,
) -> Result<Option<String>, CommandError> {
    let row: Option<(Option<String>,)> = sqlx::query_as("SELECT thumbnail_path FROM assets WHERE id = ?1")
        .bind(asset_id)
        .fetch_optional(&*db)
        .await
        .map_err(CommandError::from)?;

    if let Some((Some(path),)) = row {
        if std::path::Path::new(&path).exists() {
            return Ok(Some(path));
        }
    }
    Ok(None)
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
    let folder_lock = runtime.folder_lock(folder_id).await;
    let _folder_guard = folder_lock.lock().await;

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

#[tauri::command]
pub async fn pick_library_folder(app: AppHandle) -> Option<String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .pick_folder(move |path| {
            let _ = tx.send(path.map(|p| p.to_string().replace('\\', "/")));
        });
    rx.await.ok().flatten()
}

#[tauri::command]
pub async fn search_assets(
    db: State<'_, SqlitePool>,
    req: crate::models::AssetSearchRequest,
) -> Result<Vec<crate::models::Asset>, CommandError> {
    search::search_assets(&*db, &req).await.map_err(Into::into)
}

#[tauri::command]
pub async fn search_assets_page(
    db: State<'_, SqlitePool>,
    req: crate::models::AssetSearchRequest,
) -> Result<crate::models::AssetSearchResponse, CommandError> {
    search::search_assets_page(&*db, &req).await.map_err(Into::into)
}

#[tauri::command]
pub async fn delete_library_folder(
    db: State<'_, SqlitePool>,
    folder_id: i64,
) -> Result<bool, CommandError> {
    db::delete_library_folder(&*db, folder_id).await.map_err(CommandError::from)
}

#[tauri::command]
pub async fn open_library_folder(path: String) -> Result<(), CommandError> {
    file_actions::open_folder(&path).map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_folder_asset_counts(
    db: State<'_, SqlitePool>,
    folder_id: i64,
) -> Result<crate::models::FolderAssetCounts, CommandError> {
    let folder = db::get_folder_by_id(&*db, folder_id).await.map_err(CommandError::from)?;
    let path = folder.map(|f| f.path).unwrap_or_default();
    let (total, missing, is_accessible) = db::count_assets_by_folder(&*db, folder_id, &path).await.map_err(CommandError::from)?;
    Ok(crate::models::FolderAssetCounts { folder_id, total, missing, is_accessible })
}

#[tauri::command]
pub async fn record_recent_asset_action(
    db: State<'_, SqlitePool>,
    asset_id: i64,
    action_type: String,
) -> Result<(), CommandError> {
    match action_type.as_str() {
        "open_file" | "reveal_folder" | "copy_path" => {}
        _ => {
            return Err(CommandError {
                message: "unsupported recent action type".to_string(),
            })
        }
    }

    db::record_recent_asset_action(&*db, asset_id, &action_type)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn list_recent_asset_actions(
    db: State<'_, SqlitePool>,
    limit: i64,
) -> Result<Vec<crate::models::RecentAssetAction>, CommandError> {
    db::list_recent_asset_actions(&*db, limit)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn create_library_folder_from_path(
    db: State<'_, SqlitePool>,
    path: String,
) -> Result<crate::models::LibraryFolder, CommandError> {
    let normalized = indexer::normalize_path(std::path::Path::new(&path))
        .map_err(|e| CommandError::from(anyhow::anyhow!("failed to normalize path: {}", e)))?;

    let name = std::path::Path::new(&normalized)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let result = db::create_library_folder(&*db, &name, &normalized).await;
    if let Err(ref e) = result {
        if is_unique_constraint_error(e) {
            return db::get_folder_by_path(&*db, &normalized)
                .await
                .map_err(CommandError::from)?
                .ok_or_else(|| CommandError::from(anyhow::anyhow!("folder was not found after unique constraint")));
        }
    }
    result.map_err(CommandError::from)
}

#[tauri::command]
pub async fn asset_path_variants(
    path: String,
    project_root: Option<String>,
) -> Result<crate::models::AssetPathVariants, CommandError> {
    let forward_slash_path = path.replace('\\', "/");

    let p = std::path::Path::new(&path);
    let file_name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let folder_path = p
        .parent()
        .map(|parent| parent.to_string_lossy().to_string())
        .unwrap_or_default();

    let godot_res_path = project_root.and_then(|root| {
        let root = root.replace('\\', "/").trim_end_matches('/').to_string();
        if root.is_empty() {
            return None;
        }
        let path_normalized = path.replace('\\', "/");
        if path_normalized == root {
            Some("res://".to_string())
        } else if let Some(rest) = path_normalized.strip_prefix(&format!("{}/", root)) {
            Some(format!("res://{}", rest))
        } else {
            None
        }
    });

    Ok(crate::models::AssetPathVariants {
        absolute_path: path,
        forward_slash_path,
        folder_path,
        file_name,
        godot_res_path,
    })
}

#[cfg(test)]
mod path_variant_tests {
    use super::*;

    #[tokio::test]
    async fn derives_file_name_folder_and_forward_slash_for_windows_path() {
        let result = asset_path_variants("C:\\project\\assets\\icon.png".to_string(), None)
            .await
            .unwrap();

        assert_eq!(result.absolute_path, "C:\\project\\assets\\icon.png");
        assert_eq!(result.forward_slash_path, "C:/project/assets/icon.png");
        assert_eq!(result.folder_path, "C:\\project\\assets");
        assert_eq!(result.file_name, "icon.png");
        assert_eq!(result.godot_res_path, None);
    }

    #[tokio::test]
    async fn derives_godot_res_path_when_inside_project_root() {
        let result = asset_path_variants(
            "C:/project/assets/icon.png".to_string(),
            Some("C:/project".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(
            result.godot_res_path,
            Some("res://assets/icon.png".to_string())
        );
    }

    #[tokio::test]
    async fn godot_res_path_is_none_when_outside_project_root() {
        let result = asset_path_variants(
            "C:/other/assets/icon.png".to_string(),
            Some("C:/project".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(result.godot_res_path, None);
    }

    #[tokio::test]
    async fn godot_res_path_is_none_for_adjacent_directory() {
        let result = asset_path_variants(
            "C:/project2/assets/icon.png".to_string(),
            Some("C:/project".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(result.godot_res_path, None);
    }

    #[tokio::test]
    async fn godot_res_path_is_none_without_project_root() {
        let result = asset_path_variants(
            "C:/project/assets/icon.png".to_string(),
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.godot_res_path, None);
    }
}
