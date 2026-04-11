use std::path::Path;

use anyhow::Result;
use chrono::{Local, NaiveDate};
use console::Style;

use crate::{
    id,
    item::{Item, ItemType, Status, is_fibonacci},
    store, store_config,
};

#[derive(Debug)]
pub struct CreateArgs {
    pub title: String,
    pub item_type: ItemType,
    pub priority: u8,
    pub tags: Vec<String>,
    pub description: String,
    pub due: Option<NaiveDate>,
    pub story_points: Option<u8>,
    pub phase: String,
}

// Manual Default: priority 2 = "normal" (not 0 = "critical" which derive would give).
impl Default for CreateArgs {
    fn default() -> Self {
        Self {
            title: String::new(),
            item_type: ItemType::Task,
            priority: 2,
            tags: Vec::new(),
            description: String::new(),
            due: None,
            story_points: None,
            phase: String::new(),
        }
    }
}

/// # Errors
///
/// Returns an error if the store cannot be read, a unique ID cannot be generated,
/// or the new item cannot be written.
pub fn run(dir: &Path, args: CreateArgs) -> Result<()> {
    let description = crate::emoji::expand_shortcodes(&args.description).into_owned();
    if let Some(sp) = args.story_points
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
        title: args.title,
        status: Status::Open,
        item_type: args.item_type,
        priority: args.priority,
        tags: args.tags,
        created: today,
        updated: today,
        closed_reason: String::new(),
        dependencies: Vec::new(),
        blocks: Vec::new(),
        blocked_by: Vec::new(),
        due: args.due,
        description,
        story_points: args.story_points,
        phase: args.phase.trim().to_string(),
        resolution: String::new(),
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
