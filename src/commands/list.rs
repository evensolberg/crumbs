use std::path::Path;

use anyhow::Result;

use crate::{item::Status, store};

pub fn run(
    dir: &Path,
    status_filter: Option<&str>,
    tag_filter: Option<&str>,
    all: bool,
) -> Result<()> {
    let items = store::load_all(dir)?;
    let filtered: Vec<_> = items
        .iter()
        .filter(|(_, item)| {
            // By default hide closed items unless --all or an explicit status filter is given
            if !all && status_filter.is_none() && item.status == Status::Closed {
                return false;
            }
            if let Some(s) = status_filter {
                let parsed: Result<Status, _> = s.parse();
                if parsed.ok().as_ref() != Some(&item.status) {
                    return false;
                }
            }
            if let Some(tag) = tag_filter
                && !item.tags.iter().any(|t| t.contains(tag))
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

    for (_, item) in filtered {
        let status_icon = match item.status {
            Status::Open => "○",
            Status::InProgress => "●",
            Status::Closed => "✓",
        };
        let tags = if item.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", item.tags.join(", "))
        };
        println!(
            "{status_icon} {} [P{}] [{}] {}{tags}",
            item.id, item.priority, item.item_type, item.title
        );
    }
    Ok(())
}
