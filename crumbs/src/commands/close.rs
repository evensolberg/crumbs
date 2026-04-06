use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;

use crate::{commands::start::active_start_ts, item::Status, store};

/// # Errors
///
/// Returns an error if the item is not found or the store cannot be updated.
pub fn run(dir: &Path, id: &str, reason: Option<String>) -> Result<()> {
    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, mut item)) => {
            // cr-613: stop any running timer before closing
            if active_start_ts(&item.description).is_some() {
                super::stop::run(dir, id, None)?;
                // reload just this file after stop rewrote it
                item = store::read_item(&path)?;
            }

            let reason = reason.unwrap_or_default();

            item.status = Status::Closed;
            item.closed_reason = reason;
            item.updated = Local::now().date_naive();
            item.description.clear(); // description lives in the body, not frontmatter

            let frontmatter = serde_yaml_ng::to_string(&item)?;
            let raw = std::fs::read_to_string(&path)?;
            let body = raw
                .strip_prefix("---\n")
                .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
                .unwrap_or("");
            let new_content = format!("---\n{frontmatter}---\n{body}");
            store::atomic_write(&path, &new_content)?;

            store::reindex(dir)?;
            println!("Closed {} — {}", item.id, item.title);
        }
    }
    Ok(())
}
