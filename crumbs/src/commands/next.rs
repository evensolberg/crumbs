use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use chrono::Local;

use crate::{item::Status, store};

/// # Errors
///
/// Returns an error if the store cannot be read.
pub fn run(dir: &Path) -> Result<()> {
    let today = Local::now().date_naive();
    let items = store::load_all(dir)?;

    // Build a lookup of status by ID to check whether blockers are resolved.
    let status_by_id: HashMap<String, Status> = items
        .iter()
        .map(|(_, item)| (item.id.to_lowercase(), item.status.clone()))
        .collect();

    let candidate = items
        .into_iter()
        .filter(|(_, item)| {
            if item.status == Status::Closed {
                return false;
            }
            // Skip deferred items with a future until date.
            if item.status == Status::Deferred
                && let Some(due) = item.due
            {
                return due <= today;
            }
            // Skip items that have at least one blocker not yet closed.
            if item.blocked_by.iter().any(|id| {
                status_by_id
                    .get(&id.to_lowercase())
                    .is_some_and(|s| *s != Status::Closed)
            }) {
                return false;
            }
            true
        })
        .min_by_key(|(_, item)| (item.priority, item.created));

    match candidate {
        None => println!("No open items."),
        Some((_, item)) => {
            super::show::run(dir, &[item.id])?;
        }
    }
    Ok(())
}
