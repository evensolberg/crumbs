use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;

use chrono::NaiveDate;

use crate::{item::ItemType, store};

pub fn run(
    dir: &Path,
    id: &str,
    status: Option<String>,
    priority: Option<u8>,
    tags: Option<Vec<String>>,
    item_type: Option<String>,
    dependencies: Option<Vec<String>>,
    due: Option<NaiveDate>,
    clear_due: bool,
    message: Option<String>,
) -> Result<()> {
    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, mut item)) => {
            if let Some(s) = status {
                item.status = s.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            }
            if let Some(p) = priority {
                item.priority = p;
            }
            if let Some(t) = tags {
                item.tags = t;
            }
            if let Some(t) = item_type {
                item.item_type = t
                    .parse::<ItemType>()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
            }
            if let Some(d) = dependencies {
                item.dependencies = d;
            }
            if clear_due {
                item.due = None;
            } else if due.is_some() {
                item.due = due;
            }
            item.updated = Local::now().date_naive();

            let raw = std::fs::read_to_string(&path)?;
            let body = raw
                .strip_prefix("---\n")
                .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
                .unwrap_or("");
            let new_body = if let Some(ref msg) = message {
                item.description = msg.clone();
                if msg.is_empty() {
                    format!("\n# {}\n", item.title)
                } else {
                    format!("\n# {}\n\n{}\n", item.title, msg.trim())
                }
            } else {
                body.to_string()
            };
            let frontmatter = serde_yml::to_string(&item)?;
            let new_content = format!("---\n{frontmatter}---\n{new_body}");
            std::fs::write(&path, new_content)?;

            store::reindex(dir)?;
            println!("Updated {}", item.id);
        }
    }
    Ok(())
}
