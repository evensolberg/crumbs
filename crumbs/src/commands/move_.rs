use std::path::Path;

use anyhow::{Result, bail};

use crate::{id, store, store_config};

/// Move an item from `src_dir` to `dst_dir`.
///
/// A new ID is generated using the destination store's prefix. The original
/// file is deleted and both stores are reindexed.
pub fn run(src_dir: &Path, id: &str, dst_dir: &Path) -> Result<()> {
    if !dst_dir.is_dir() {
        bail!(
            "destination store not found: {} — run: crumbs init",
            dst_dir.display()
        );
    }

    let (src_path, mut item) = store::find_by_id(src_dir, id)?
        .ok_or_else(|| anyhow::anyhow!("no item found with id: {id}"))?;

    let old_id = item.id.clone();

    // Generate a new ID in the destination store.
    let dst_prefix = store_config::load(dst_dir).prefix;
    let dst_items = store::load_all(dst_dir)?;
    let new_id = id::generate(&dst_prefix, |candidate| {
        dst_items
            .iter()
            .any(|(_, i)| i.id.eq_ignore_ascii_case(candidate))
    })?;

    item.id = new_id.clone();

    // Write to destination then remove the source file.
    store::write_item(dst_dir, &item)?;
    std::fs::remove_file(&src_path)?;

    store::reindex(src_dir)?;
    store::reindex(dst_dir)?;

    println!("Moved {} → {} ({})", old_id, new_id, dst_dir.display());
    Ok(())
}
