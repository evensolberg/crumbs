use std::path::PathBuf;

use crumbs::{
    commands::{
        clean, close, create, create::CreateArgs, delete, export, link, start, stop, update,
        update::UpdateArgs,
    },
    config::global_dir,
    item::{Item, Status},
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
    update::run(
        &to_path(&dir),
        &id,
        UpdateArgs {
            status: Some(status),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}

/// Update an item's priority (0–4).
#[tauri::command]
pub fn update_priority(dir: String, id: String, priority: u8) -> Result<(), String> {
    update::run(
        &to_path(&dir),
        &id,
        UpdateArgs {
            priority: Some(priority),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}

/// Update an item's type.
#[tauri::command]
pub fn update_type(dir: String, id: String, item_type: String) -> Result<(), String> {
    update::run(
        &to_path(&dir),
        &id,
        UpdateArgs {
            item_type: Some(item_type),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}

/// Update an item's due date. Empty string clears the due date.
#[tauri::command]
pub fn update_due(dir: String, id: String, due: String) -> Result<(), String> {
    if due.is_empty() {
        update::run(
            &to_path(&dir),
            &id,
            UpdateArgs {
                clear_due: true,
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())
    } else {
        let date = due
            .parse::<chrono::NaiveDate>()
            .map_err(|e| e.to_string())?;
        update::run(
            &to_path(&dir),
            &id,
            UpdateArgs {
                due: Some(date),
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())
    }
}

/// Update the markdown body text of an item.
#[tauri::command]
pub fn update_body(dir: String, id: String, body: String) -> Result<(), String> {
    update::run(
        &to_path(&dir),
        &id,
        UpdateArgs {
            message: Some(body),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}

/// Update an item's story points. Value 0 clears the points.
#[tauri::command]
pub fn update_points(dir: String, id: String, points: u8) -> Result<(), String> {
    if points == 0 {
        update::run(
            &to_path(&dir),
            &id,
            UpdateArgs {
                clear_points: true,
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())
    } else {
        update::run(
            &to_path(&dir),
            &id,
            UpdateArgs {
                story_points: Some(points),
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())
    }
}

/// Update an item's dependencies. Empty string clears all dependencies.
#[tauri::command]
pub fn update_dependencies(dir: String, id: String, dependencies: String) -> Result<(), String> {
    let dep_list: Vec<String> = if dependencies.is_empty() {
        vec![]
    } else {
        dependencies
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    update::run(
        &to_path(&dir),
        &id,
        UpdateArgs {
            dependencies: Some(dep_list),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}

/// Update an item's phase label. Empty or whitespace-only string clears the phase.
/// Uses `clear_phase: true` (not `phase: Some("")`) so the clear intent
/// remains correct if validation is added to the `phase` assignment path later.
#[tauri::command]
pub fn update_phase(dir: String, id: String, phase: String) -> Result<(), String> {
    let phase = phase.trim().to_string();
    if phase.is_empty() {
        update::run(
            &to_path(&dir),
            &id,
            UpdateArgs {
                clear_phase: true,
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())
    } else {
        update::run(
            &to_path(&dir),
            &id,
            UpdateArgs {
                phase: Some(phase),
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())
    }
}

/// Update an item's tags. Empty string clears all tags.
#[tauri::command]
pub fn update_tags(dir: String, id: String, tags: String) -> Result<(), String> {
    let tag_list: Vec<String> = if tags.is_empty() {
        vec![]
    } else {
        tags.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    update::run(
        &to_path(&dir),
        &id,
        UpdateArgs {
            tags: Some(tag_list),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}

/// Update an item's title.
#[tauri::command]
pub fn update_title(dir: String, id: String, title: String) -> Result<(), String> {
    update::run(
        &to_path(&dir),
        &id,
        UpdateArgs {
            title: Some(title),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}

/// Update an item's resolution reference (PR/commit URL or identifier).
#[tauri::command]
pub fn update_resolution(dir: String, id: String, resolution: String) -> Result<(), String> {
    update::run(
        &to_path(&dir),
        &id,
        UpdateArgs {
            resolution: Some(resolution),
            ..Default::default()
        },
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
        CreateArgs {
            title,
            ..Default::default()
        },
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

/// Move an item from the current store to a different store.
/// Returns the new ID assigned in the destination store.
#[tauri::command]
pub fn move_item(src_dir: String, id: String, dst_dir: String) -> Result<(), String> {
    let src = to_path(&src_dir);
    let dst = to_path(&dst_dir);
    crumbs::commands::move_::run(&src, &id, &dst).map_err(|e| e.to_string())
}

/// Defer an item, optionally with a wake-up date (YYYY-MM-DD). Empty string = no date.
#[tauri::command]
pub fn defer_item(dir: String, id: String, until: String) -> Result<(), String> {
    let until_date = if until.is_empty() {
        None
    } else {
        Some(
            until
                .parse::<chrono::NaiveDate>()
                .map_err(|e| e.to_string())?,
        )
    };
    crumbs::commands::defer::run(&to_path(&dir), &id, false, until_date).map_err(|e| e.to_string())
}

/// Full-text search across item titles and raw file content.
#[tauri::command]
pub fn search_items(dir: String, query: String, include_closed: bool) -> Result<Vec<Item>, String> {
    let path = to_path(&dir);
    let items = store::load_all(&path).map_err(|e| e.to_string())?;
    let q = query.to_lowercase();
    let results = items
        .into_iter()
        .filter_map(|(file_path, item)| {
            if !include_closed && matches!(item.status, Status::Closed) {
                return None;
            }
            let raw = std::fs::read_to_string(&file_path).unwrap_or_default();
            if raw.to_lowercase().contains(&q) {
                Some(item)
            } else {
                None
            }
        })
        .collect();
    Ok(results)
}

/// Export all items to the given format (json, csv, toon) and return content as a string.
#[tauri::command]
pub fn export_items(dir: String, format: String) -> Result<String, String> {
    export::to_string(&to_path(&dir), &format).map_err(|e| e.to_string())
}

/// Write text content to a file at the given absolute path.
#[tauri::command]
pub fn write_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

/// Rebuild the CSV index from .md files on disk.
#[tauri::command]
pub fn reindex_store(dir: String) -> Result<(), String> {
    store::reindex(&to_path(&dir)).map_err(|e| e.to_string())
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

/// Start a timer for an item (appends [start] entry, sets status to in_progress).
#[tauri::command]
pub fn start_timer(dir: String, id: String, comment: String) -> Result<(), String> {
    let c = if comment.is_empty() {
        None
    } else {
        Some(comment.as_str())
    };
    start::run(&to_path(&dir), &id, c).map_err(|e| e.to_string())
}

/// Stop the active timer for an item (appends [stop] entry with elapsed time).
#[tauri::command]
pub fn stop_timer(dir: String, id: String, comment: String) -> Result<(), String> {
    let c = if comment.is_empty() {
        None
    } else {
        Some(comment.as_str())
    };
    stop::run(&to_path(&dir), &id, c).map_err(|e| e.to_string())
}
