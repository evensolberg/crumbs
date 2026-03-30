use std::path::Path;

use anyhow::{Result, bail};
use chrono::{Local, NaiveDate};

use crate::{item::Status, store};

/// `crumbs defer <id> [--until <date>]` — set status to deferred, optionally setting a wake-up date.
/// `crumbs defer <id> --reopen` — set status back to open.
///
/// # Errors
///
/// Returns an error if the item is not found or the store cannot be updated.
pub fn run(dir: &Path, id: &str, reopen: bool, until: Option<NaiveDate>) -> Result<()> {
    let (path, mut item) = store::find_by_id(dir, id)?
        .ok_or_else(|| anyhow::anyhow!("no item found with id: {id}"))?;

    if reopen {
        if item.status != Status::Deferred {
            bail!("{} is not deferred (status: {})", item.id, item.status);
        }
        item.status = Status::Open;
        item.updated = Local::now().date_naive();
        store::rewrite_frontmatter(&path, &item)?;
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
        store::rewrite_frontmatter(&path, &item)?;
        store::reindex(dir)?;
        println!("Deferred {}", item.id);
    }
    Ok(())
}
