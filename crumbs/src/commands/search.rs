use std::path::Path;

use anyhow::Result;
use chrono::Local;
use console::measure_text_width;

use crate::{commands::row::format_row, store};

/// # Errors
///
/// Returns an error if the store cannot be read.
pub fn run(dir: &Path, query: &str) -> Result<()> {
    let items = store::load_all(dir)?;
    let q = query.to_lowercase();
    let matches: Vec<_> = items
        .into_iter()
        .filter(|(_, item)| {
            let lq = &q;
            item.title.to_lowercase().contains(lq)
                || item.description.to_lowercase().contains(lq)
                || item.id.to_lowercase().contains(lq)
                || item.phase.to_lowercase().contains(lq)
                || item.item_type.to_string().to_lowercase().contains(lq)
                || item.status.to_string().to_lowercase().contains(lq)
                || item.tags.iter().any(|t| t.to_lowercase().contains(lq))
                || item
                    .due
                    .is_some_and(|d| d.to_string().contains(lq.as_str()))
        })
        .collect();

    if matches.is_empty() {
        println!("No items found matching '{query}'.");
        return Ok(());
    }

    // Compute each phase's display width once, then derive max for column alignment.
    let matches_with_widths: Vec<_> = matches
        .into_iter()
        .map(|(p, i)| {
            let w = measure_text_width(&i.phase);
            (p, i, w)
        })
        .collect();
    let max_phase = matches_with_widths
        .iter()
        .map(|(_, _, w)| *w)
        .max()
        .unwrap_or(0);
    let spaces = " ".repeat(max_phase);
    let today = Local::now().date_naive();

    for (_, item, display_w) in matches_with_widths {
        let padding = max_phase.saturating_sub(display_w);
        let phase_badge = format!("[{}{}]", item.phase, &spaces[..padding]);
        println!("{}", format_row(&item, &phase_badge, today));
    }
    Ok(())
}
