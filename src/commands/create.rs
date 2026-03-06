use std::path::Path;

use anyhow::Result;
use chrono::Local;

use chrono::NaiveDate;

use crate::{
    id,
    item::{Item, ItemType, Status},
    store, store_config,
};

pub fn run(
    dir: &Path,
    title: String,
    item_type: ItemType,
    priority: u8,
    tags: Vec<String>,
    description: String,
    dependencies: Vec<String>,
    due: Option<NaiveDate>,
) -> Result<()> {
    let today = Local::now().date_naive();
    let prefix = store_config::load(dir).prefix;
    // Collect existing IDs so we can guarantee uniqueness.
    let existing_ids: std::collections::HashSet<String> = store::load_all(dir)
        .unwrap_or_default()
        .into_iter()
        .map(|(_, i)| i.id.to_lowercase())
        .collect();
    let item = Item {
        id: id::generate(&prefix, |candidate| {
            existing_ids.contains(&candidate.to_lowercase())
        })?,
        title,
        status: Status::Open,
        item_type,
        priority,
        tags,
        created: today,
        updated: today,
        closed_reason: String::new(),
        dependencies,
        blocks: Vec::new(),
        blocked_by: Vec::new(),
        due,
        description,
    };
    let path = store::write_item(dir, &item)?;
    store::reindex(dir)?;
    println!("Created {} — {}", item.id, item.title);
    println!("  {}", path.display());
    Ok(())
}
