use std::path::Path;

use anyhow::{Result, bail};

use crate::store;

pub fn run(dir: &Path, id: &str) -> Result<()> {
    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, item)) => {
            println!("{} — {}", item.id, item.title);
            println!("  Status:   {}", item.status);
            println!("  Type:     {}", item.item_type);
            println!("  Priority: P{}", item.priority);
            if !item.tags.is_empty() {
                println!("  Tags:     {}", item.tags.join(", "));
            }
            println!("  Created:  {}", item.created);
            println!("  Updated:  {}", item.updated);
            if !item.closed_reason.is_empty() {
                println!("  Closed:   {}", item.closed_reason);
            }
            if !item.dependencies.is_empty() {
                println!("  Deps:     {}", item.dependencies.join(", "));
            }
            if !item.description.is_empty() {
                println!();
                println!("{}", item.description);
                println!();
            }
            println!("  File:     {}", path.display());
        }
    }
    Ok(())
}
