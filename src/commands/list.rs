use std::path::Path;

use anyhow::Result;

use crate::{color, item::Status, store};

pub fn run(
    dir: &Path,
    status_filter: Option<&str>,
    tag_filter: Option<&str>,
    priority_filter: Option<u8>,
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

    for (_, item) in filtered {
        let icon = color::status_icon_styled(&item.status);
        let p_style = color::priority(item.priority);
        let t_style = color::item_type(&item.item_type);
        let tags = if item.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", item.tags.join(", "))
        };
        println!(
            "{icon} {} {} {} {}{tags}",
            item.id,
            p_style.apply_to(format!("[P{}]", item.priority)),
            t_style.apply_to(format!("[{}]", item.item_type)),
            item.title
        );
    }
    Ok(())
}
