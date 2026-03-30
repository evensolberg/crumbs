use std::path::Path;

use anyhow::Result;

use crate::store;

/// # Errors
///
/// Returns an error if the store cannot be read or the index cannot be rebuilt.
pub fn run(dir: &Path) -> Result<()> {
    let items = store::load_all(dir)?;
    store::reindex(dir)?;
    println!("Reindexed {} items → index.csv", items.len());
    Ok(())
}
