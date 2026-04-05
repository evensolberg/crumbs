use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{Local, NaiveDate};
use console::Style;

use crate::{
    color,
    commands::start::active_start_ts,
    item::{Item, ItemType, Status},
    store,
};

/// Fields by which `crumbs list` output can be sorted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum SortKey {
    Id,
    Priority,
    Status,
    Title,
    Type,
    Due,
    Created,
    Updated,
}

impl std::fmt::Display for SortKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Id => "id",
            Self::Priority => "priority",
            Self::Status => "status",
            Self::Title => "title",
            Self::Type => "type",
            Self::Due => "due",
            Self::Created => "created",
            Self::Updated => "updated",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for SortKey {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "id" => Ok(Self::Id),
            "priority" => Ok(Self::Priority),
            "status" => Ok(Self::Status),
            "title" => Ok(Self::Title),
            "type" => Ok(Self::Type),
            "due" => Ok(Self::Due),
            "created" => Ok(Self::Created),
            "updated" => Ok(Self::Updated),
            other => Err(format!(
                "unknown sort key {other:?}; valid: id, priority, status, title, type, due, created, updated"
            )),
        }
    }
}

/// Sort a list of `(path, item)` pairs by the given key.
#[must_use]
pub fn sort_items(mut items: Vec<(PathBuf, Item)>, key: SortKey) -> Vec<(PathBuf, Item)> {
    match key {
        SortKey::Id => items.sort_by(|a, b| a.1.id.cmp(&b.1.id)),
        SortKey::Priority => {
            items.sort_by_cached_key(|(_, i)| (i.priority, i.id.clone()));
        }
        SortKey::Status => {
            items.sort_by_cached_key(|(_, i)| (i.status.to_string(), i.id.clone()));
        }
        SortKey::Title => {
            items.sort_by_cached_key(|(_, i)| (i.title.to_lowercase(), i.id.clone()));
        }
        SortKey::Type => {
            items.sort_by_cached_key(|(_, i)| (i.item_type.to_string(), i.id.clone()));
        }
        // Treat None as "no due date" — sort to the end, after all dated items.
        SortKey::Due => {
            items.sort_by_cached_key(|(_, i)| (i.due.unwrap_or(NaiveDate::MAX), i.id.clone()));
        }
        SortKey::Created => {
            items.sort_by_cached_key(|(_, i)| (i.created, i.id.clone()));
        }
        SortKey::Updated => {
            items.sort_by_cached_key(|(_, i)| (i.updated, i.id.clone()));
        }
    }
    items
}

/// Arguments for `crumbs list`.
#[derive(Default)]
pub struct ListArgs {
    pub status_filter: Option<String>,
    pub tag_filter: Option<String>,
    pub priority_filter: Option<u8>,
    pub type_filter: Option<ItemType>,
    pub all: bool,
    pub verbose: bool,
    pub sort: Option<SortKey>,
}

/// # Errors
///
/// Returns an error if the status filter value is not a recognised [`Status`]
/// variant, or if the store directory cannot be read.
pub fn run(dir: &Path, args: ListArgs) -> Result<()> {
    let ListArgs {
        status_filter,
        tag_filter,
        priority_filter,
        type_filter,
        all,
        verbose,
        sort,
    } = args;
    let sort = sort.unwrap_or(SortKey::Id);
    // Validate the status filter up front so a typo surfaces as an error
    // rather than silently returning "No items found."
    let status_filter_parsed: Option<Status> = match status_filter.as_deref() {
        None => None,
        Some(s) => Some(
            s.parse()
                .map_err(|e: String| anyhow::anyhow!("invalid --status value: {e}"))?,
        ),
    };

    // Parse comma-separated tag filter once before iteration.
    // AND semantics: all non-empty parts must each match at least one tag.
    // Empty parts (e.g. trailing comma) are ignored so "--tag alpha," == "--tag alpha".
    let tag_parts: Option<Vec<&str>> = tag_filter.as_deref().and_then(|s| {
        let parts: Vec<&str> = s
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
        if parts.is_empty() { None } else { Some(parts) }
    });

    let items = store::load_all(dir)?;
    let filtered: Vec<_> = items
        .into_iter()
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
            if let Some(parts) = &tag_parts {
                if !parts
                    .iter()
                    .all(|req| item.tags.iter().any(|t| t.contains(req)))
                {
                    return false;
                }
            }
            if let Some(p) = priority_filter
                && item.priority != p
            {
                return false;
            }
            if let Some(ref t) = type_filter
                && &item.item_type != t
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

    let sorted = sort_items(filtered, sort);
    let today = Local::now().date_naive();
    for (_, item) in sorted {
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
        let points_marker = item
            .story_points
            .map_or_else(String::new, |sp| format!(" [{sp}sp]"));
        let timer_marker = if active_start_ts(&item.description).is_some() {
            " ▶"
        } else {
            ""
        };
        println!(
            "{icon} {} {} {} {}{timer_marker}{tags}{due_marker}{points_marker}",
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
