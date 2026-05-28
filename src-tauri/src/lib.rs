mod commands;
mod db;
mod file_actions;
mod indexer;
mod models;
mod tags;
mod thumbnails;

use std::path::PathBuf;
use tauri::Manager;

#[derive(Clone)]
pub struct ThumbnailDir(pub PathBuf);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            let db_path = app_dir.join("data.sqlite");
            let pool = tauri::async_runtime::block_on(async {
                db::connect(&db_path).await.map_err(|e| {
                    eprintln!("Database connection failed: {}", e);
                    e
                })
            })?;
            app.manage(pool);

            let thumbnail_dir = app_dir.join("thumbnails");
            if let Err(e) = std::fs::create_dir_all(&thumbnail_dir) {
                eprintln!("Failed to create thumbnail directory: {}", e);
            }
            app.manage(ThumbnailDir(thumbnail_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_assets,
            commands::list_library_folders,
            commands::set_asset_favorite,
            commands::apply_tag_to_assets,
            commands::open_asset_file,
            commands::reveal_asset_in_folder,
            commands::add_library_folder,
            commands::scan_library_folder,
            commands::list_asset_tags
        ])
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
