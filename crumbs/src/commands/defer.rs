use std::path::Path;

use anyhow::{Result, bail};
use chrono::{Local, NaiveDate};

use crate::{item::Status, store};

fn update_item_file(path: &std::path::PathBuf, item: &crate::item::Item) -> Result<()> {
    let mut fm = item.clone();
    fm.description.clear(); // description lives in the body, not frontmatter
    let frontmatter = serde_yaml_ng::to_string(&fm)?;
    let raw = std::fs::read_to_string(path)?;
    let body = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
        .unwrap_or("");
    store::atomic_write(path, &format!("---\n{frontmatter}---\n{body}"))?;
    Ok(())
}

/// `crumbs defer <id> [--until <date>]` — set status to deferred, optionally setting a wake-up date.
/// `crumbs defer <id> --reopen` — set status back to open.
pub fn run(dir: &Path, id: &str, reopen: bool, until: Option<NaiveDate>) -> Result<()> {
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
        if let Some(date) = until {
            item.due = Some(date);
        }
        item.updated = Local::now().date_naive();
        update_item_file(&path, &item)?;
        store::reindex(dir)?;
        println!("Deferred {}", item.id);
    }
    Ok(())
}
