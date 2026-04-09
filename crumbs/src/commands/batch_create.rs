use std::path::Path;

use anyhow::Result;
use chrono::{Local, NaiveDate};
use console::Style;
use serde::Deserialize;

use crate::{
    id,
    item::{Item, ItemType, Status, is_fibonacci},
    store, store_config,
};

/// A single item spec for batch creation. All fields except `title` are optional
/// and default to the same values as `crumbs create` with no flags.
#[derive(Debug, Deserialize)]
pub struct BatchCreateItem {
    pub title: String,
    #[serde(rename = "type", default)]
    pub item_type: ItemType,
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Body text (equivalent to --message on the CLI).
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub due: Option<NaiveDate>,
    pub story_points: Option<u8>,
    #[serde(default)]
    pub phase: String,
}

const fn default_priority() -> u8 {
    2
}

impl Default for BatchCreateItem {
    fn default() -> Self {
        Self {
            title: String::new(),
            item_type: ItemType::Task,
            priority: 2,
            tags: Vec::new(),
            message: String::new(),
            dependencies: Vec::new(),
            due: None,
            story_points: None,
            phase: String::new(),
        }
    }
}

/// Create multiple items in one operation.
///
/// IDs are generated fresh for every item; uniqueness is guaranteed across the
/// entire batch and against items already in the store. `store::reindex` is
/// called once at the end rather than once per item.
///
/// # Errors
///
/// Returns an error if any item has an invalid `story_points` value, if a
/// unique ID cannot be generated, or if a write fails.
pub fn run(dir: &Path, items: Vec<BatchCreateItem>) -> Result<()> {
    if items.is_empty() {
        return Ok(());
    }

    for item in &items {
        if let Some(sp) = item.story_points
            && !is_fibonacci(sp)
        {
            anyhow::bail!(
                "story_points must be a Fibonacci number (1, 2, 3, 5, 8, 13, 21); got {sp} in \"{}\"",
                item.title
            );
        }
    }

    let today = Local::now().date_naive();
    let prefix = store_config::load(dir).prefix;
    // Seed the ID collision set with items already in the store. We grow it
    // as we generate IDs so each new ID is unique within the batch too.
    let mut used_ids: std::collections::HashSet<String> = store::load_all(dir)
        .unwrap_or_default()
        .into_iter()
        .map(|(_, i)| i.id.to_lowercase())
        .collect();

    let bold = Style::new().bold();
    for spec in items {
        let description = crate::emoji::expand_shortcodes(&spec.message).into_owned();
        let new_id = id::generate(&prefix, |candidate| {
            used_ids.contains(&candidate.to_lowercase())
        })?;
        used_ids.insert(new_id.to_lowercase());

        let item = Item {
            id: new_id,
            title: spec.title,
            status: Status::Open,
            item_type: spec.item_type,
            priority: spec.priority,
            tags: spec.tags,
            created: today,
            updated: today,
            closed_reason: String::new(),
            dependencies: spec.dependencies,
            blocks: Vec::new(),
            blocked_by: Vec::new(),
            due: spec.due,
            description,
            story_points: spec.story_points,
            phase: spec.phase.trim().to_string(),
            resolution: String::new(),
        };
        let path = store::write_item(dir, &item)?;
        println!("Created {} — {}", bold.apply_to(&item.id), item.title);
        println!("  {}", path.display());
    }

    store::reindex(dir)?;
    Ok(())
}

/// Parse a JSON or YAML byte slice into a `Vec<BatchCreateItem>` and run batch
/// creation.
///
/// `format` must be `"json"` or `"yaml"`.
///
/// # Errors
///
/// Returns an error if the format is unrecognised, the input is malformed, or
/// any item fails to create.
pub fn run_from_slice(dir: &Path, bytes: &[u8], format: &str) -> Result<()> {
    let items: Vec<BatchCreateItem> = match format {
        "json" => serde_json::from_slice(bytes)?,
        "yaml" => serde_yaml_ng::from_slice(bytes)?,
        other => anyhow::bail!("unsupported batch-create format: {other} (expected json or yaml)"),
    };
    run(dir, items)
}

/// Infer the batch-create format from the file extension, falling back to
/// `explicit` when provided.
///
/// Recognised extensions: `.json`, `.yaml`, `.yml`.
/// Returns an error for any other extension (or no extension) when `explicit`
/// is `None`.
///
/// # Errors
///
/// Returns an error if the format cannot be inferred and `explicit` is `None`.
pub fn infer_format<'a>(path: &Path, explicit: Option<&'a str>) -> Result<&'a str> {
    if let Some(f) = explicit {
        return Ok(f);
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => Ok("json"),
        Some("yaml" | "yml") => Ok("yaml"),
        Some(ext) => anyhow::bail!(
            "cannot infer batch-create format from .{ext}; use --format json or --format yaml"
        ),
        None => anyhow::bail!("file has no extension; use --format json or --format yaml"),
    }
}
