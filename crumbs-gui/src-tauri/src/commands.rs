use std::path::PathBuf;

use crumbs::{
    commands::{clean, close, create, delete, link, update},
    config::global_dir,
    item::{Item, ItemType},
    store, store_config,
};

fn to_path(dir: &str) -> PathBuf {
    if dir == "global" {
        global_dir()
    } else {
        PathBuf::from(dir)
    }
}

/// Walk up from `start` looking for a `.crumbs/` directory.
/// Returns the first one found, or `None` if we reach the filesystem root.
fn find_crumbs_upward(start: &std::path::Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        let candidate = cur.join(".crumbs");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// Resolve the store directory from an optional explicit path string.
/// When `dir` is empty, walks up from cwd looking for `.crumbs/`,
/// then falls back to the global store.
#[tauri::command]
pub fn resolve_store(dir: String) -> Result<String, String> {
    let path = if dir.is_empty() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        find_crumbs_upward(&cwd).unwrap_or_else(global_dir)
    } else {
        to_path(&dir)
    };
    Ok(path.to_string_lossy().into_owned())
}

/// Return all items from the store (excluding closed by default).
#[tauri::command]
pub fn list_items(dir: String, include_closed: bool) -> Result<Vec<Item>, String> {
    let path = to_path(&dir);
    let items = store::load_all(&path).map_err(|e| e.to_string())?;
    let filtered = items
        .into_iter()
        .map(|(_, item)| item)
        .filter(|item| include_closed || !matches!(item.status, crumbs::item::Status::Closed))
        .collect();
    Ok(filtered)
}

/// Return a single item by ID.
#[tauri::command]
pub fn get_item(dir: String, id: String) -> Result<Option<Item>, String> {
    let path = to_path(&dir);
    let result = store::find_by_id(&path, &id)
        .map_err(|e| e.to_string())?
        .map(|(_, item)| item);
    Ok(result)
}

/// Update an item's status.
#[tauri::command]
pub fn update_status(dir: String, id: String, status: String) -> Result<(), String> {
    let path = to_path(&dir);
    update::run(
        &path,
        &id,
        Some(status),
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        false,
        None,
    )
    .map_err(|e| e.to_string())
}

/// Update an item's priority (0–4).
#[tauri::command]
pub fn update_priority(dir: String, id: String, priority: u8) -> Result<(), String> {
    let path = to_path(&dir);
    update::run(
        &path,
        &id,
        None,
        Some(priority),
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        false,
        None,
    )
    .map_err(|e| e.to_string())
}

/// Update an item's type.
#[tauri::command]
pub fn update_type(dir: String, id: String, item_type: String) -> Result<(), String> {
    let path = to_path(&dir);
    update::run(
        &path,
        &id,
        None,
        None,
        None,
        Some(item_type),
        None,
        None,
        false,
        None,
        None,
        false,
        None,
    )
    .map_err(|e| e.to_string())
}

/// Update an item's due date. Empty string clears the due date.
#[tauri::command]
pub fn update_due(dir: String, id: String, due: String) -> Result<(), String> {
    let path = to_path(&dir);
    if due.is_empty() {
        update::run(
            &path, &id, None, None, None, None, None, None, true, None, None, false, None,
        )
        .map_err(|e| e.to_string())
    } else {
        let date = due
            .parse::<chrono::NaiveDate>()
            .map_err(|e| e.to_string())?;
        update::run(
            &path,
            &id,
            None,
            None,
            None,
            None,
            None,
            Some(date),
            false,
            None,
            None,
            false,
            None,
        )
        .map_err(|e| e.to_string())
    }
}

/// Update the markdown body text of an item.
#[tauri::command]
pub fn update_body(dir: String, id: String, body: String) -> Result<(), String> {
    let path = to_path(&dir);
    update::run(
        &path,
        &id,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        Some(body),
        None,
        false,
        None,
    )
    .map_err(|e| e.to_string())
}

/// Update an item's story points. Value 0 clears the points.
#[tauri::command]
pub fn update_points(dir: String, id: String, points: u8) -> Result<(), String> {
    let path = to_path(&dir);
    if points == 0 {
        update::run(
            &path, &id, None, None, None, None, None, None, false, None, None, true, None,
        )
        .map_err(|e| e.to_string())
    } else {
        update::run(
            &path,
            &id,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
            None,
            Some(points),
            false,
            None,
        )
        .map_err(|e| e.to_string())
    }
}

/// Update an item's title.
#[tauri::command]
pub fn update_title(dir: String, id: String, title: String) -> Result<(), String> {
    let path = to_path(&dir);
    update::run(
        &path,
        &id,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        false,
        Some(title),
    )
    .map_err(|e| e.to_string())
}

/// Close an item with an optional reason.
#[tauri::command]
pub fn close_item(dir: String, id: String, reason: String) -> Result<(), String> {
    let path = to_path(&dir);
    let reason_opt = if reason.is_empty() {
        None
    } else {
        Some(reason)
    };
    close::run(&path, &id, reason_opt).map_err(|e| e.to_string())
}

/// Check whether a directory has an existing .crumbs store.
/// Returns true if:
///   - the path itself is a store (contains index.csv or config.toml), OR
///   - the path contains a .crumbs/ subdirectory (project store)
#[tauri::command]
pub fn has_store(dir: String) -> bool {
    let p = PathBuf::from(&dir);
    p.join("index.csv").is_file() || p.join("config.toml").is_file() || p.join(".crumbs").is_dir()
}

/// Initialize a .crumbs store in the given directory with a derived prefix.
#[tauri::command]
pub fn init_store(dir: String) -> Result<String, String> {
    let base = PathBuf::from(&dir);
    let crumbs_dir = base.join(".crumbs");
    // Derive a prefix from the directory name.
    let prefix = store_config::suggest_prefix(&base);
    crumbs::commands::init::run(&crumbs_dir, Some(prefix)).map_err(|e| e.to_string())?;
    Ok(crumbs_dir.to_string_lossy().into_owned())
}

/// Create a new item with the given title and default settings.
#[tauri::command]
pub fn create_item(dir: String, title: String) -> Result<(), String> {
    let path = to_path(&dir);
    create::run(
        &path,
        title,
        ItemType::Task,
        2,
        vec![],
        String::new(),
        vec![],
        None,
        None,
    )
    .map_err(|e| e.to_string())
}

/// Delete a single item by ID.
#[tauri::command]
pub fn delete_item(dir: String, id: String) -> Result<(), String> {
    let path = to_path(&dir);
    delete::run(&path, &id).map_err(|e| e.to_string())
}

/// Remove all closed items from the store.
#[tauri::command]
pub fn clean_closed(dir: String) -> Result<(), String> {
    let path = to_path(&dir);
    clean::run(&path).map_err(|e| e.to_string())
}

/// Link or unlink items. relation is "blocks" or "blocked-by".
/// Sets/clears blocked_by and blocks on both sides atomically.
#[tauri::command]
pub fn link_items(
    dir: String,
    id: String,
    relation: String,
    targets: Vec<String>,
    remove: bool,
) -> Result<(), String> {
    let path = to_path(&dir);
    link::run(&path, &id, &relation, &targets, remove).map_err(|e| e.to_string())
}
