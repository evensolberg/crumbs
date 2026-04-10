use std::path::PathBuf;

use anyhow::Result;

use crate::item::{Item, ItemType, Status};

/// Arguments that select a subset of items from the store.
///
/// Used by [`crate::commands::list`], [`crate::commands::update::run_bulk`],
/// and [`crate::commands::close::run_bulk`].
#[derive(Default)]
pub struct FilterArgs {
    /// Filter by status (raw string, validated inside [`apply`]).
    pub status: Option<String>,
    /// Filter by tag (comma-separated). AND semantics: every non-empty part
    /// must match at least one tag on the item.
    pub tag: Option<String>,
    /// Filter by exact priority value.
    pub priority: Option<u8>,
    /// Filter by item type.
    pub r#type: Option<ItemType>,
    /// Filter by phase (exact, after trimming whitespace).
    pub phase: Option<String>,
    /// When `true`, include closed items even when no explicit `status` filter
    /// is set. Mirrors the `--all` flag on `crumbs list`.
    pub all: bool,
}

/// Apply `args` to `items` and return the matching subset.
///
/// Filtering is AND-combined: an item must satisfy **all** non-`None` criteria.
/// Closed items are hidden by default unless `args.all` is `true` or
/// `args.status` is set.
///
/// # Errors
///
/// Returns an error if `args.status` is present but is not a valid [`Status`]
/// string.
pub fn apply(items: Vec<(PathBuf, Item)>, args: &FilterArgs) -> Result<Vec<(PathBuf, Item)>> {
    // Validate and parse the status filter once before iterating.
    let status_parsed: Option<Status> = match args.status.as_deref() {
        None => None,
        Some(s) => Some(
            s.parse()
                .map_err(|e: String| anyhow::anyhow!("invalid status filter value: {e}"))?,
        ),
    };

    // Parse comma-separated tag filter once.
    // AND semantics: every non-empty part must appear in at least one tag.
    // Empty parts (trailing commas, etc.) are ignored.
    let tag_parts: Option<Vec<&str>> = args.tag.as_deref().and_then(|s| {
        let parts: Vec<&str> = s
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
        if parts.is_empty() { None } else { Some(parts) }
    });

    // Trim phase filter once; an all-whitespace value collapses to "no filter".
    let phase = args
        .phase
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let filtered = items
        .into_iter()
        .filter(|(_, item)| {
            // By default, hide closed items unless --all or an explicit status
            // filter is given (mirrors list.rs behaviour exactly).
            if !args.all && status_parsed.is_none() && item.status == Status::Closed {
                return false;
            }
            if status_parsed.as_ref().is_some_and(|s| s != &item.status) {
                return false;
            }
            if tag_parts.as_ref().is_some_and(|parts| {
                !parts
                    .iter()
                    .all(|req| item.tags.iter().any(|t| t.contains(req)))
            }) {
                return false;
            }
            if args.priority.is_some_and(|p| item.priority != p) {
                return false;
            }
            if args.r#type.as_ref().is_some_and(|t| &item.item_type != t) {
                return false;
            }
            if phase.is_some_and(|p| item.phase != p) {
                return false;
            }
            true
        })
        .collect();

    Ok(filtered)
}
