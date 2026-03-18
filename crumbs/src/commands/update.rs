use std::path::Path;

use anyhow::{Result, bail};
use chrono::{Local, NaiveDate};

use crate::{
    item::{ItemType, is_fibonacci},
    store,
};

#[derive(Default)]
pub struct UpdateArgs {
    pub status: Option<String>,
    pub priority: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub item_type: Option<String>,
    pub dependencies: Option<Vec<String>>,
    pub due: Option<NaiveDate>,
    pub clear_due: bool,
    pub message: Option<String>,
    pub append: bool,
    pub story_points: Option<u8>,
    pub clear_points: bool,
    pub title: Option<String>,
    /// Override the verb printed on success (default: "Updated")
    pub output_label: Option<String>,
}

pub fn run(dir: &Path, id: &str, args: UpdateArgs) -> Result<()> {
    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, mut item)) => {
            if let Some(s) = args.status {
                item.status = s.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            }
            if let Some(p) = args.priority {
                item.priority = p;
            }
            if let Some(t) = args.tags {
                item.tags = t;
            }
            if let Some(t) = args.item_type {
                item.item_type = t
                    .parse::<ItemType>()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
            }
            if let Some(d) = args.dependencies {
                item.dependencies = d;
            }
            if let Some(t) = args.title {
                let t = t.trim().to_string();
                if !t.is_empty() {
                    item.title = t;
                }
            }
            if args.clear_due {
                item.due = None;
            } else if args.due.is_some() {
                item.due = args.due;
            }
            if args.clear_points {
                item.story_points = None;
            } else if let Some(sp) = args.story_points {
                if !is_fibonacci(sp) {
                    anyhow::bail!(
                        "story_points must be a Fibonacci number (1, 2, 3, 5, 8, 13, 21); got {sp}"
                    );
                }
                item.story_points = Some(sp);
            }
            item.updated = Local::now().date_naive();

            let raw = std::fs::read_to_string(&path)?;
            let body = raw
                .strip_prefix("---\n")
                .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
                .unwrap_or("");
            // Extract the existing description (everything after the heading line).
            let existing_desc = {
                let trimmed = body.trim_start_matches('\n');
                trimmed
                    .split_once('\n')
                    .map(|(_, rest)| rest.trim_matches('\n'))
                    .unwrap_or("")
                    .to_string()
            };
            // Build the new description:
            // - append mode: timestamp + new text appended after existing content
            // - replace mode: new message replaces existing content
            // - no message: preserve existing content (heading still updated for title renames)
            let desc = match &args.message {
                Some(msg) if args.append => {
                    let timestamp = Local::now().format("%Y-%m-%d");
                    if existing_desc.is_empty() {
                        format!("[{timestamp}] {}", msg.trim())
                    } else {
                        format!("{}\n\n[{timestamp}] {}", existing_desc, msg.trim())
                    }
                }
                Some(msg) => msg.trim().to_string(),
                None => existing_desc,
            };
            let desc = crate::emoji::expand_shortcodes(&desc).into_owned();
            let new_body = if desc.is_empty() {
                format!("\n# {}\n", item.title)
            } else {
                format!("\n# {}\n\n{}\n", item.title, desc)
            };
            item.description.clear(); // description lives in the body, not frontmatter
            let frontmatter = serde_yaml_ng::to_string(&item)?;
            let new_content = format!("---\n{frontmatter}---\n{new_body}");
            store::atomic_write(&path, &new_content)?;

            store::reindex(dir)?;
            let label = args.output_label.as_deref().unwrap_or("Updated");
            println!("{label} {}", item.id);
        }
    }
    Ok(())
}
