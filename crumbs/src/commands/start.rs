use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;

use crate::{item::Status, store};

/// Returns the start timestamp ("YYYY-MM-DD HH:MM:SS") if an unmatched
/// `[start]` entry exists in the description.
pub fn active_start_ts(description: &str) -> Option<String> {
    let mut last: Option<String> = None;
    for line in description.lines() {
        let t = line.trim();
        if t.starts_with("[start]") {
            let rest = t.trim_start_matches("[start]").trim();
            last = Some(rest[..rest.len().min(19)].to_string());
        } else if t.starts_with("[stop]") {
            last = None;
        }
    }
    last
}

pub fn run(dir: &Path, id: &str, comment: Option<&str>) -> Result<()> {
    let (path, mut item) = store::find_by_id(dir, id)?
        .ok_or_else(|| anyhow::anyhow!("no item found with id: {id}"))?;

    let raw = std::fs::read_to_string(&path)?;
    let body = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, b)| b))
        .unwrap_or("");
    let existing_desc = {
        let trimmed = body.trim_start_matches('\n');
        trimmed
            .split_once('\n')
            .map(|(_, rest)| rest.trim_matches('\n'))
            .unwrap_or("")
            .to_string()
    };

    if let Some(ts) = active_start_ts(&existing_desc) {
        bail!("Already started at {ts}");
    }

    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S");
    let entry = match comment {
        Some(c) if !c.trim().is_empty() => {
            let c = crate::emoji::expand_shortcodes(c.trim());
            format!("[start] {timestamp}  {c}")
        }
        _ => format!("[start] {timestamp}"),
    };
    let desc = if existing_desc.is_empty() {
        entry
    } else {
        format!("{existing_desc}\n\n{entry}")
    };

    item.status = Status::InProgress;
    item.updated = now.date_naive();
    item.description.clear(); // description lives in the body, not frontmatter

    let new_body = format!("\n# {}\n\n{}\n", item.title, desc);
    let frontmatter = serde_yaml_ng::to_string(&item)?;
    let new_content = format!("---\n{frontmatter}---\n{new_body}");
    store::atomic_write(&path, &new_content)?;
    store::reindex(dir)?;

    println!("Started timer for {} — {}", item.id, item.title);
    Ok(())
}
