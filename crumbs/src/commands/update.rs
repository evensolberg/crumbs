use std::path::Path;

use anyhow::{Result, bail};
use chrono::{Local, NaiveDate};

use crate::{
    item::{Item, ItemType, Status, is_fibonacci},
    store,
};

/// Apply a new status string to `item`.
///
/// If the item is transitioning from `closed` to any other status and has a
/// non-empty `closed_reason`, the reason is moved into a timestamped reopen
/// note (returned as `Some(note)`) and `closed_reason` is cleared. Otherwise
/// returns `None`.
///
/// # Errors
///
/// Returns an error if `new_status` is not a valid status string.
fn apply_status(item: &mut Item, new_status: &str) -> Result<Option<String>> {
    let status: Status = new_status.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    let reopen_note = if matches!(item.status, Status::Closed)
        && !matches!(status, Status::Closed)
        && !item.closed_reason.is_empty()
    {
        let timestamp = Local::now().format("%Y-%m-%d");
        let note = format!(
            "[{timestamp}] Reopened (was closed: {})",
            item.closed_reason
        );
        item.closed_reason.clear();
        Some(note)
    } else {
        None
    };
    item.status = status;
    Ok(reopen_note)
}

// UpdateArgs is a plain argument bag. The four bool fields are semantically
// distinct clear-flags (not state-machine transitions), so the lint does not
// apply here.
#[allow(clippy::struct_excessive_bools)]
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
    pub phase: Option<String>,
    pub clear_phase: bool,
    pub resolution: Option<String>,
}

/// Update an item. Prints `"Updated <id>"` on success.
///
/// # Errors
///
/// Returns an error if the item is not found or the store cannot be updated.
pub fn run(dir: &Path, id: &str, args: UpdateArgs) -> Result<()> {
    run_labeled(dir, id, args, None)
}

/// Like [`run`], but overrides the success verb (e.g. `"Appended to"`).
/// Used by the CLI `append` subcommand; not intended for library consumers.
#[doc(hidden)]
#[allow(clippy::too_many_lines)] // one branch per updatable field; no natural split point
pub fn run_labeled(
    dir: &Path,
    id: &str,
    args: UpdateArgs,
    output_label: Option<&str>,
) -> Result<()> {
    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, mut item)) => {
            let reopen_note = if let Some(s) = args.status {
                apply_status(&mut item, &s)?
            } else {
                None
            };
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
            if args.clear_phase {
                item.phase = String::new();
            } else if let Some(p) = args.phase {
                item.phase = p.trim().to_string();
            }
            if let Some(r) = args.resolution {
                item.resolution = r.trim().to_string();
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
                    .map_or("", |(_, rest)| rest.trim_matches('\n'))
                    .to_string()
            };
            // Build the new description:
            // - append mode: timestamp + new text appended after existing content
            // - replace mode: new message replaces existing content
            // - no message: preserve existing content (heading still updated for title renames)
            // - reopen_note (if any) is always appended after existing content.
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
            let desc = if let Some(note) = reopen_note {
                if desc.is_empty() {
                    note
                } else {
                    format!("{desc}\n\n{note}")
                }
            } else {
                desc
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
            let label = output_label.unwrap_or("Updated");
            println!("{label} {}", item.id);
        }
    }
    Ok(())
}
