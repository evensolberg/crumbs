use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;

use crate::{item::Status, store};

fn update_item_file(path: &std::path::PathBuf, item: &crate::item::Item) -> Result<()> {
    let frontmatter = serde_yaml_ng::to_string(item)?;
    let raw = std::fs::read_to_string(path)?;
    let body = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
        .unwrap_or("");
    store::atomic_write(path, &format!("---\n{frontmatter}---\n{body}"))?;
    Ok(())
}

/// `crumbs defer <id>` — set status to deferred.
/// `crumbs defer <id> --reopen` — set status back to open.
pub fn run(dir: &Path, id: &str, reopen: bool) -> Result<()> {
    let (path, mut item) = store::find_by_id(dir, id)?
        .ok_or_else(|| anyhow::anyhow!("no item found with id: {id}"))?;

    if reopen {
        if item.status != Status::Deferred {
            bail!("{} is not deferred (status: {})", item.id, item.status);
        }
        item.status = Status::Open;
        item.updated = Local::now().date_naive();
        update_item_file(&path, &item)?;
        store::reindex(dir)?;
        println!("Reopened {}", item.id);
    } else {
        if item.status == Status::Deferred {
            bail!("{} is already deferred", item.id);
        }
        item.status = Status::Deferred;
        item.updated = Local::now().date_naive();
        update_item_file(&path, &item)?;
        store::reindex(dir)?;
        println!("Deferred {}", item.id);
    }
    Ok(())
}
