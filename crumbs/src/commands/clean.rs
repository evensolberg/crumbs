use std::path::Path;

use anyhow::Result;

use crate::{item::Status, store};

pub fn run(dir: &Path) -> Result<()> {
    let items = store::load_all(dir)?;
    let closed: Vec<_> = items
        .into_iter()
        .filter(|(_, i)| i.status == Status::Closed)
        .collect();

    if closed.is_empty() {
        println!("No closed items to remove.");
        return Ok(());
    }

    for (path, item) in &closed {
        std::fs::remove_file(path)?;
        println!("Deleted {} — {}", item.id, item.title);
    }
    store::reindex(dir)?;
    println!("Removed {} closed item(s).", closed.len());
    Ok(())
}
