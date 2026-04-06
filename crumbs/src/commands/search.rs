use std::path::Path;

use anyhow::Result;
use chrono::Local;

use crate::{
    commands::row::{PhaseColumn, format_row},
    store,
};

/// # Errors
///
/// Returns an error if the store cannot be read.
pub fn run(dir: &Path, query: &str) -> Result<()> {
    let items = store::load_all(dir)?;
    let q = query.to_lowercase();
    let matches: Vec<_> = items
        .into_iter()
        .filter(|(_, item)| {
            item.title.to_lowercase().contains(&q)
                || item.description.to_lowercase().contains(&q)
                || item.id.to_lowercase().contains(&q)
                || item.phase.to_lowercase().contains(&q)
                || item.item_type.to_string().to_lowercase().contains(&q)
                || item.status.to_string().to_lowercase().contains(&q)
                || item.tags.iter().any(|t| t.to_lowercase().contains(&q))
                || item.due.is_some_and(|d| d.to_string().contains(q.as_str()))
        })
        .collect();

    if matches.is_empty() {
        println!("No items found matching '{query}'.");
        return Ok(());
    }

    let phase_col = PhaseColumn::new(matches.iter().map(|(_, i)| i.phase.as_str()));
    let today = Local::now().date_naive();

    for (_, item) in &matches {
        println!("{}", format_row(item, &phase_col.badge(&item.phase), today));
    }
    Ok(())
}
