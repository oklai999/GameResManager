mod commands;
mod db;
mod file_actions;
mod indexer;
mod models;
mod tags;
mod thumbnails;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_assets,
            commands::list_library_folders,
            commands::set_asset_favorite,
            commands::apply_tag_to_assets,
            commands::open_asset_file,
            commands::reveal_asset_in_folder
        ])
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
