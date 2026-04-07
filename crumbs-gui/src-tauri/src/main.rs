// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::resolve_store,
            commands::list_items,
            commands::get_item,
            commands::update_status,
            commands::update_priority,
            commands::update_type,
            commands::update_due,
            commands::update_body,
            commands::update_points,
            commands::update_dependencies,
            commands::update_tags,
            commands::update_phase,
            commands::update_title,
            commands::update_resolution,
            commands::close_item,
            commands::has_store,
            commands::init_store,
            commands::create_item,
            commands::delete_item,
            commands::clean_closed,
            commands::link_items,
            commands::move_item,
            commands::defer_item,
            commands::search_items,
            commands::export_items,
            commands::write_text_file,
            commands::reindex_store,
            commands::start_timer,
            commands::stop_timer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
