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

impl UpdateArgs {
    /// Returns `true` if at least one field mutation is requested.
    ///
    /// Used by [`run_bulk`] to guard against a no-op bulk update that would
    /// silently bump `updated` on every matched item.
    #[must_use]
    pub fn has_any_mutation(&self) -> bool {
        self.status.is_some()
            || self.priority.is_some()
            || self.tags.is_some()
            || self.item_type.is_some()
            || self.dependencies.is_some()
            || self.due.is_some()
            || self.clear_due
            || self.message.is_some()
            || self.story_points.is_some()
            || self.clear_points
            // apply_update silently ignores empty/whitespace titles, so only
            // count title as a mutation when there is actual content to write.
            || self.title.as_deref().is_some_and(|t| !t.trim().is_empty())
            || self.phase.is_some()
            || self.clear_phase
            || self.resolution.is_some()
    }
}

/// Update an item. Prints `"Updated <id>"` on success.
///
/// # Errors
///
/// Returns an error if the item is not found or the store cannot be updated.
pub fn run(dir: &Path, id: &str, args: UpdateArgs) -> Result<()> {
    run_labeled(dir, id, args, None)
}

/// Apply all mutations from `args` to `item` and build the new file content.
///
/// `raw` is the current on-disk content of the item's markdown file (used to
/// extract the existing body). Returns the complete new file content ready for
/// [`store::atomic_write`].
///
/// # Errors
///
/// Returns an error if `args.status`, `args.item_type`, or `args.story_points`
/// contains an invalid value.
fn apply_update(item: &mut Item, raw: &str, args: &UpdateArgs) -> Result<String> {
    let reopen_note = if let Some(s) = args.status.as_deref() {
        apply_status(item, s)?
    } else {
        None
    };
    if let Some(p) = args.priority {
        item.priority = p;
    }
    if let Some(t) = &args.tags {
        item.tags.clone_from(t);
    }
    if let Some(t) = args.item_type.as_deref() {
        item.item_type = t
            .parse::<ItemType>()
            .map_err(|e: String| anyhow::anyhow!(e))?;
    }
    if let Some(d) = &args.dependencies {
        item.dependencies.clone_from(d);
    }
    if let Some(t) = args.title.as_deref() {
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
    } else if let Some(p) = args.phase.as_deref() {
        item.phase = p.trim().to_string();
    }
    if let Some(r) = args.resolution.as_deref() {
        item.resolution = r.trim().to_string();
    }
    item.updated = Local::now().date_naive();

    let body = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
        .unwrap_or("");
    let existing_desc = {
        let trimmed = body.trim_start_matches('\n');
        trimmed
            .split_once('\n')
            .map_or("", |(_, rest)| rest.trim_matches('\n'))
            .to_string()
    };
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
    item.description.clear();
    let frontmatter = serde_yaml_ng::to_string(item)?;
    Ok(format!("---\n{frontmatter}---\n{new_body}"))
}

/// Like [`run`], but overrides the success verb (e.g. `"Appended to"`).
/// Used by the CLI `append` subcommand; not intended for library consumers.
#[doc(hidden)]
#[allow(clippy::needless_pass_by_value)] // public API: callers pass by value; avoid breaking change
pub fn run_labeled(
    dir: &Path,
    id: &str,
    args: UpdateArgs,
    output_label: Option<&str>,
) -> Result<()> {
    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, mut item)) => {
            let raw = std::fs::read_to_string(&path)?;
            let new_content = apply_update(&mut item, &raw, &args)?;
            store::atomic_write(&path, &new_content)?;
            store::reindex(dir)?;
            let label = output_label.unwrap_or("Updated");
            println!("{label} {}", item.id);
        }
    }
    Ok(())
}

/// Arguments for [`run_bulk`].
pub struct BulkUpdateArgs {
    /// Criteria for selecting items to update.
    pub filter: crate::commands::filter::FilterArgs,
    /// The mutations to apply to each matching item.
    pub update: UpdateArgs,
    /// When `true`, print what would be updated without writing any changes.
    pub dry_run: bool,
}

/// Apply `args.update` to every item matching `args.filter`.
///
/// Prints `"Updated <id>"` per item (or `"Would update <id> — <title>"` in
/// dry-run mode) and a final summary line. Calls [`store::reindex`] once at
/// the end (not per item).
///
/// Returns `Ok(())` (with a "No items matched." message) when the filter
/// matches nothing.
///
/// # Errors
///
/// Returns an error if the filter is invalid or any store write fails.
#[allow(clippy::needless_pass_by_value)] // intentional: callers construct and pass args by value
pub fn run_bulk(dir: &Path, args: BulkUpdateArgs) -> Result<()> {
    if !args.dry_run && !args.update.has_any_mutation() {
        anyhow::bail!(
            "no update fields specified — provide at least one field to change\n\
             (e.g. --status, --priority, --tags, --message, --phase, …)"
        );
    }

    let items = store::load_all(dir)?;
    let matched = crate::commands::filter::apply(items, &args.filter)?;

    if matched.is_empty() {
        println!("No items matched.");
        return Ok(());
    }

    let count = matched.len();

    if args.dry_run {
        for (_, item) in &matched {
            println!("Would update {} — {}", item.id, item.title);
        }
        println!("{count} item(s) would be updated.");
        return Ok(());
    }

    for (path, mut item) in matched {
        let raw = std::fs::read_to_string(&path)?;
        let new_content = apply_update(&mut item, &raw, &args.update)?;
        store::atomic_write(&path, &new_content)?;
        println!("Updated {}", item.id);
    }

    store::reindex(dir)?;
    println!("Updated {count} item(s).");
    Ok(())
}
