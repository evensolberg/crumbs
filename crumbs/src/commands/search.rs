use std::path::Path;

use anyhow::Result;
use chrono::Local;
use console::{Style, measure_text_width};

use crate::{color, commands::start::active_start_ts, store};

/// # Errors
///
/// Returns an error if the store cannot be read.
pub fn run(dir: &Path, query: &str) -> Result<()> {
    let items = store::load_all(dir)?;
    let q = query.to_lowercase();
    let mut matches: Vec<_> = items
        .into_iter()
        .filter(|(path, item)| {
            item.title.to_lowercase().contains(&q)
                || std::fs::read_to_string(path)
                    .map(|s| s.to_lowercase().contains(&q))
                    .unwrap_or(false)
        })
        .collect();

    if matches.is_empty() {
        println!("No items found matching '{query}'.");
        return Ok(());
    }

    // Compute phase display widths once; derive max for column alignment.
    let phase_widths: Vec<usize> = matches
        .iter()
        .map(|(_, i)| measure_text_width(&i.phase))
        .collect();
    let max_phase = phase_widths.iter().copied().max().unwrap_or(0);
    let spaces = " ".repeat(max_phase);
    let today = Local::now().date_naive();

    for ((_, item), display_w) in matches.into_iter().zip(phase_widths) {
        let icon = color::status_icon_styled(&item.status);
        let p_style = color::priority(item.priority);
        let t_style = color::item_type(&item.item_type);
        let padding = max_phase.saturating_sub(display_w);
        let phase_badge = format!("[{}{}]", item.phase, &spaces[..padding]);
        let tags = if item.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", item.tags.join(", "))
        };
        let due_marker = match item.due {
            Some(d) if d < today => format!(" {}", Style::new().red().bold().apply_to("!due")),
            Some(d) => format!(" due:{d}"),
            None => String::new(),
        };
        let points_marker = item
            .story_points
            .map_or_else(String::new, |sp| format!(" [{sp}sp]"));
        let timer_marker = if active_start_ts(&item.description).is_some() {
            " ▶"
        } else {
            ""
        };
        println!(
            "{icon} {} {} {} {} {}{timer_marker}{tags}{due_marker}{points_marker}",
            item.id,
            p_style.apply_to(format!("[P{}]", item.priority)),
            phase_badge,
            t_style.apply_to(format!("[{}]", item.item_type)),
            item.title,
        );
    }
    Ok(())
}
