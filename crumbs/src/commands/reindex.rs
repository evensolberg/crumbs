use std::path::Path;

use anyhow::Result;

use crate::store;

pub fn run(dir: &Path) -> Result<()> {
    let items = store::load_all(dir)?;
    store::reindex(dir)?;
    println!("Reindexed {} items → index.csv", items.len());
    Ok(())
}
