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
