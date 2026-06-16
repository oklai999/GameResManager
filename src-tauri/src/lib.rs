mod commands;
mod db;
mod file_actions;
mod indexer;
pub mod media;
mod models;
mod scan_service;
mod search;
mod tags;
mod thumbnails;

use std::path::PathBuf;
use tauri::Manager;

use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ThumbnailDir(pub PathBuf);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .register_asynchronous_uri_scheme_protocol(
            "asset-media",
            |context, request, responder| {
                let pool = match context.app_handle().try_state::<SqlitePool>() {
                    Some(pool) => pool.inner().clone(),
                    None => {
                        responder.respond(
                            tauri::http::Response::builder()
                                .status(503)
                                .header(
                                    tauri::http::header::CONTENT_TYPE,
                                    "text/plain; charset=utf-8",
                                )
                                .header(tauri::http::header::CONTENT_LENGTH, 21)
                                .body("Service Unavailable".as_bytes().to_vec())
                                .unwrap(),
                        );
                        return;
                    }
                };
                tauri::async_runtime::spawn(async move {
                    let response = media::handle_protocol_request(&pool, request).await;
                    responder.respond(response);
                });
            },
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            let db_path = db::resolve_database_path(&app_dir)?;
            let pool = tauri::async_runtime::block_on(async {
                db::connect(&db_path).await.map_err(|e| {
                    eprintln!("Database connection failed: {}", e);
                    e
                })
            })?;
            let ensure_pool = pool.clone();
            let cleanup_pool = pool.clone();
            let activity_cleanup_pool = pool.clone();
            app.manage(pool);

            let thumbnail_dir = app_dir.join("thumbnails");
            if let Err(e) = std::fs::create_dir_all(&thumbnail_dir) {
                eprintln!("Failed to create thumbnail directory: {}", e);
            }
            app.manage(ThumbnailDir(thumbnail_dir.clone()));
            app.manage(scan_service::ScanRuntime::default());

            let db_path_str = db_path.to_string_lossy().to_string();
            let thumb_dir_str = thumbnail_dir.to_string_lossy().to_string();
            tauri::async_runtime::block_on(async {
                db::ensure_app_paths(&ensure_pool, &db_path_str, &thumb_dir_str).await
            })?;

            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::cleanup_old_scan_jobs(&cleanup_pool, 10).await {
                    eprintln!("Failed to cleanup old scan jobs: {}", e);
                }
            });

            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::cleanup_recent_asset_actions(&activity_cleanup_pool).await {
                    eprintln!("Failed to cleanup recent activity: {}", e);
                }
            });

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
            commands::pick_library_folder,
            commands::create_library_folder_from_path,
            commands::delete_library_folder,
            commands::list_asset_tags,
            commands::list_tags,
            commands::list_recent_tags,
            commands::get_asset_tags,
            commands::list_common_tags,
            commands::update_tag,
            commands::remove_tag_from_assets,
            commands::delete_tag,
            commands::list_collections,
            commands::create_collection,
            commands::add_assets_to_collection,
            commands::remove_asset_from_collection,
            commands::update_collection,
            commands::remove_assets_from_collection,
            commands::delete_collection,
            commands::list_collection_assets,
            commands::asset_thumbnail_url,
            commands::update_asset_note,
            commands::start_scan,
            commands::cancel_scan,
            commands::latest_scan_job,
            commands::get_scan_settings,
            commands::save_scan_settings,
            commands::search_assets,
            commands::search_assets_page,
            commands::open_library_folder,
            commands::get_folder_asset_counts,
            commands::record_recent_asset_action,
            commands::list_recent_activity,
            commands::asset_path_variants
        ])
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
