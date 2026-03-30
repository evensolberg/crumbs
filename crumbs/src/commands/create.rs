use std::path::Path;

use anyhow::Result;
use chrono::{Local, NaiveDate};
use console::Style;

use crate::{
    id,
    item::{Item, ItemType, Status, is_fibonacci},
    store, store_config,
};

/// # Errors
///
/// Returns an error if the store cannot be read, a unique ID cannot be generated,
/// or the new item cannot be written.
pub fn run(
    dir: &Path,
    title: String,
    item_type: ItemType,
    priority: u8,
    tags: Vec<String>,
    description: String,
    dependencies: Vec<String>,
    due: Option<NaiveDate>,
    story_points: Option<u8>,
) -> Result<()> {
    let description = crate::emoji::expand_shortcodes(&description).into_owned();
    if let Some(sp) = story_points
        && !is_fibonacci(sp)
    {
        anyhow::bail!("story_points must be a Fibonacci number (1, 2, 3, 5, 8, 13, 21); got {sp}");
    }
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
        story_points,
    };
    let path = store::write_item(dir, &item)?;
    store::reindex(dir)?;
    println!(
        "Created {} — {}",
        Style::new().bold().apply_to(&item.id),
        item.title
    );
    println!("  {}", path.display());
    Ok(())
}
