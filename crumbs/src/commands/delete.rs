use std::path::Path;

use anyhow::{Result, bail};

use crate::{item::Status, store};

pub fn run(dir: &Path, id: &str) -> Result<()> {
    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, item)) => {
            std::fs::remove_file(&path)?;
            store::reindex(dir)?;
            println!("Deleted {} — {}", item.id, item.title);
        }
    }
    Ok(())
}

pub fn run_closed(dir: &Path) -> Result<()> {
    let items = store::load_all(dir)?;
    let closed: Vec<_> = items
        .into_iter()
        .filter(|(_, i)| i.status == Status::Closed)
        .collect();

    if closed.is_empty() {
        println!("No closed items to delete.");
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
