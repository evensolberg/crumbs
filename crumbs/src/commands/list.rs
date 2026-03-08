use std::path::Path;

use anyhow::Result;
use chrono::Local;
use console::Style;

use crate::{color, item::Status, store};

pub fn run(
    dir: &Path,
    status_filter: Option<&str>,
    tag_filter: Option<&str>,
    priority_filter: Option<u8>,
    all: bool,
    verbose: bool,
) -> Result<()> {
    // Validate the status filter up front so a typo surfaces as an error
    // rather than silently returning "No items found."
    let status_filter_parsed: Option<Status> = match status_filter {
        None => None,
        Some(s) => Some(
            s.parse()
                .map_err(|e: String| anyhow::anyhow!("invalid --status value: {e}"))?,
        ),
    };

    let items = store::load_all(dir)?;
    let filtered: Vec<_> = items
        .iter()
        .filter(|(_, item)| {
            // By default hide closed items unless --all or an explicit status filter is given.
            // Blocked and deferred items remain visible by default.
            if !all && status_filter_parsed.is_none() && item.status == Status::Closed {
                return false;
            }
            if status_filter_parsed
                .as_ref()
                .is_some_and(|s| s != &item.status)
            {
                return false;
            }
            if let Some(tag) = tag_filter
                && !item.tags.iter().any(|t| t.contains(tag))
            {
                return false;
            }
            if let Some(p) = priority_filter
                && item.priority != p
            {
                return false;
            }
            true
        })
        .collect();

    if filtered.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    let today = Local::now().date_naive();
    for (_, item) in filtered {
        let icon = color::status_icon_styled(&item.status);
        let p_style = color::priority(item.priority);
        let t_style = color::item_type(&item.item_type);
        let tags = if item.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", item.tags.join(", "))
        };
        let due_marker = match item.due {
            Some(d) if d < today => {
                format!(" {}", Style::new().red().bold().apply_to("!due"))
            }
            Some(d) => format!(" due:{d}"),
            None => String::new(),
        };
        let points_marker = match item.story_points {
            Some(sp) => format!(" [{sp}sp]"),
            None => String::new(),
        };
        println!(
            "{icon} {} {} {} {}{tags}{due_marker}{points_marker}",
            item.id,
            p_style.apply_to(format!("[P{}]", item.priority)),
            t_style.apply_to(format!("[{}]", item.item_type)),
            item.title
        );
        if verbose && !item.description.is_empty() {
            let snippet = item
                .description
                .lines()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ");
            println!("  {}", Style::new().dim().apply_to(snippet));
        }
    }
    Ok(())
}
