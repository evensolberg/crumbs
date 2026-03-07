use std::path::Path;

use anyhow::Result;

use crate::store;

pub fn run(dir: &Path, query: &str) -> Result<()> {
    let items = store::load_all(dir)?;
    let q = query.to_lowercase();
    let mut found = 0;

    for (path, item) in &items {
        let raw = std::fs::read_to_string(path)?;
        if raw.to_lowercase().contains(&q) || item.title.to_lowercase().contains(&q) {
            println!("{} [{}] {}", item.id, item.item_type, item.title);
            found += 1;
        }
    }

    if found == 0 {
        println!("No items found matching '{query}'.");
    }
    Ok(())
}
