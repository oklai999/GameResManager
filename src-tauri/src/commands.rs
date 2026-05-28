use crate::{db, file_actions};
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

#[tauri::command]
pub async fn list_assets(db: State<'_, SqlitePool>) -> Result<Vec<crate::models::Asset>, CommandError> {
    db::list_assets(&db, 200, 0).await.map_err(Into::into)
}

#[tauri::command]
pub async fn list_library_folders(
    db: State<'_, SqlitePool>,
) -> Result<Vec<crate::models::LibraryFolder>, CommandError> {
    db::list_library_folders(&db).await.map_err(Into::into)
}

#[tauri::command]
pub async fn set_asset_favorite(
    db: State<'_, SqlitePool>,
    asset_id: i64,
    is_favorite: bool,
) -> Result<(), CommandError> {
    db::set_asset_favorite(&db, asset_id, is_favorite).await.map_err(Into::into)
}

#[tauri::command]
pub async fn apply_tag_to_assets(
    db: State<'_, SqlitePool>,
    tag_name: String,
    asset_ids: Vec<i64>,
) -> Result<(), CommandError> {
    db::apply_tag_to_assets(&db, &tag_name, &asset_ids).await.map_err(Into::into)
}

#[tauri::command]
pub async fn open_asset_file(path: String) -> Result<(), CommandError> {
    file_actions::open_file(&path).map_err(Into::into)
}

#[tauri::command]
pub async fn reveal_asset_in_folder(path: String) -> Result<(), CommandError> {
    file_actions::reveal_in_folder(&path).map_err(Into::into)
}
