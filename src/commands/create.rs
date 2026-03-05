use std::path::Path;

use anyhow::Result;
use chrono::Local;

use crate::{
    id,
    item::{Item, ItemType, Status},
    store,
};

pub fn run(
    dir: &Path,
    title: String,
    item_type: ItemType,
    priority: u8,
    tags: Vec<String>,
) -> Result<()> {
    let today = Local::now().date_naive();
    let item = Item {
        id: id::generate(),
        title,
        status: Status::Open,
        item_type,
        priority,
        tags,
        created: today,
        updated: today,
        closed_reason: String::new(),
        dependencies: Vec::new(),
    };
    let path = store::write_item(dir, &item)?;
    store::reindex(dir)?;
    println!("Created {} — {}", item.id, item.title);
    println!("  {}", path.display());
    Ok(())
}
