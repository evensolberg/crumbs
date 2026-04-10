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
            // cr-613: stop any running timer before closing.
            // Use run_no_reindex because close::run calls reindex below anyway.
            if active_start_ts(&item.description).is_some() {
                super::stop::run_no_reindex(dir, id, None)?;
                // stop wrote the file; item is now stale — reload from disk
                item = store::read_item(&path)?;
            }

            let reason = reason.unwrap_or_default();

            item.status = Status::Closed;
            item.closed_reason = reason;
            item.updated = Local::now().date_naive();
            // Clear before rewrite_frontmatter to avoid cloning a potentially
            // large description string inside that function.
            item.description.clear();

            store::rewrite_frontmatter(&path, &item)?;

            store::reindex(dir)?;
            println!("Closed {} — {}", item.id, item.title);
        }
    }
    Ok(())
}

/// Close every item matching `filter`.
///
/// For each matched item: stops any active timer (preserves time-log
/// accuracy), then sets `status = Closed` with the optional `reason`. Prints
/// `"Closed <id> — <title>"` per item and a final summary.
///
/// When `dry_run` is `true`, prints what would be closed without writing any
/// changes.
///
/// Returns `Ok(())` (with a "No items matched." message) when the filter
/// matches nothing.
///
/// # Errors
///
/// Returns an error if the filter is invalid or any store write fails.
#[allow(clippy::needless_pass_by_value)] // intentional: callers construct and pass filter by value
pub fn run_bulk(
    dir: &Path,
    filter: crate::commands::filter::FilterArgs,
    reason: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let items = store::load_all(dir)?;
    let matched = crate::commands::filter::apply(items, &filter)?;

    if matched.is_empty() {
        println!("No items matched.");
        return Ok(());
    }

    if dry_run {
        let mut count = 0usize;
        for (_, item) in &matched {
            if item.status == Status::Closed {
                continue;
            }
            println!("Would close {} — {}", item.id, item.title);
            count += 1;
        }
        if count == 0 {
            println!("No items to close (all matched items were already closed).");
        } else {
            println!("{count} item(s) would be closed.");
        }
        return Ok(());
    }

    let reason_str = reason.unwrap_or_default();
    let mut count = 0usize;

    for (path, mut item) in matched {
        // Already closed — skip to keep bulk-close idempotent and avoid
        // overwriting a pre-existing closed_reason with an empty string.
        if item.status == Status::Closed {
            continue;
        }

        // stop::run rewrites the file on disk, making the in-memory item stale;
        // reload after to ensure subsequent writes are based on current content.
        if active_start_ts(&item.description).is_some() {
            // Use run_no_reindex to avoid one extra reindex per item with an
            // active timer; run_bulk calls reindex once after the loop.
            super::stop::run_no_reindex(dir, &item.id, None)?;
            item = store::read_item(&path)?;
        }

        item.status = Status::Closed;
        item.closed_reason.clone_from(&reason_str);
        item.updated = Local::now().date_naive();
        // Clear before rewrite_frontmatter to avoid cloning a potentially
        // large description string inside that function.
        item.description.clear();

        store::rewrite_frontmatter(&path, &item)?;
        println!("Closed {} — {}", item.id, item.title);
        count += 1;
    }

    if count == 0 {
        println!("No items to close (all matched items were already closed).");
        return Ok(());
    }

    store::reindex(dir)?;
    println!("Closed {count} item(s).");
    Ok(())
}
