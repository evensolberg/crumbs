use std::path::Path;

use anyhow::Result;
use chrono::Local;

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
) -> Result<()> {
    let today = Local::now().date_naive();
    let prefix = store_config::load(dir).prefix;
    let item = Item {
        id: id::generate(&prefix),
        title,
        status: Status::Open,
        item_type,
        priority,
        tags,
        created: today,
        updated: today,
        closed_reason: String::new(),
        dependencies,
        description,
    };
    let path = store::write_item(dir, &item)?;
    store::reindex(dir)?;
    println!("Created {} — {}", item.id, item.title);
    println!("  {}", path.display());
    Ok(())
}
