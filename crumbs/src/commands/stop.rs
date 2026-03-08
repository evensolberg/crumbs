use std::path::Path;

use anyhow::Result;
use chrono::{Local, NaiveDateTime};

use crate::store;

pub fn format_elapsed(secs: i64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
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

    let start_ts = super::start::active_start_ts(&existing_desc)
        .ok_or_else(|| anyhow::anyhow!("No active timer for {id}"))?;

    let start_dt = NaiveDateTime::parse_from_str(&start_ts, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| anyhow::anyhow!("could not parse start timestamp '{start_ts}': {e}"))?;

    let now = Local::now();
    let elapsed_secs = (now.naive_local() - start_dt).num_seconds().max(0);
    let elapsed = format_elapsed(elapsed_secs);
    let timestamp = now.format("%Y-%m-%d %H:%M:%S");

    let entry = match comment {
        Some(c) if !c.trim().is_empty() => {
            let c = crate::emoji::expand_shortcodes(c.trim());
            format!("[stop]  {timestamp}  {elapsed}  {c}")
        }
        _ => format!("[stop]  {timestamp}  {elapsed}"),
    };
    let desc = if existing_desc.is_empty() {
        entry
    } else {
        format!("{existing_desc}\n\n{entry}")
    };

    item.updated = now.date_naive();
    item.description.clear(); // description lives in the body, not frontmatter

    let new_body = format!("\n# {}\n\n{}\n", item.title, desc);
    let frontmatter = serde_yaml_ng::to_string(&item)?;
    let new_content = format!("---\n{frontmatter}---\n{new_body}");
    store::atomic_write(&path, &new_content)?;
    store::reindex(dir)?;

    println!(
        "Stopped timer for {} — {} (elapsed: {elapsed})",
        item.id, item.title
    );
    Ok(())
}
