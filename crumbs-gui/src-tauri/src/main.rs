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
            commands::update_title,
            commands::close_item,
            commands::has_store,
            commands::init_store,
            commands::create_item,
            commands::delete_item,
            commands::clean_closed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
