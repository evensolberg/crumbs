use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{Local, NaiveDate};
use console::Style;

use crate::{
    commands::row::{PhaseColumn, format_row},
    item::{Item, ItemType},
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
    Phase,
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
            Self::Phase => "phase",
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
            "phase" => Ok(Self::Phase),
            other => Err(format!(
                "unknown sort key {other:?}; valid: id, priority, status, title, type, due, created, updated, phase"
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
        // Items without a phase sort to the end; within a phase, sort by id.
        // The bool tuple makes the "empty last" intent explicit and correct for
        // all Unicode phase labels (is_empty=false < is_empty=true).
        SortKey::Phase => {
            items.sort_by_cached_key(|(_, i)| {
                (i.phase.is_empty(), i.phase.to_lowercase(), i.id.clone())
            });
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
    pub phase_filter: Option<String>,
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
        phase_filter,
        all,
        verbose,
        sort,
    } = args;
    let sort = sort.unwrap_or(SortKey::Id);

    let items = store::load_all(dir)?;
    let filtered = crate::commands::filter::apply(
        items,
        &crate::commands::filter::FilterArgs {
            status: status_filter,
            tag: tag_filter,
            priority: priority_filter,
            r#type: type_filter,
            phase: phase_filter,
            all,
        },
    )?;

    if filtered.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    let sorted = sort_items(filtered, sort);
    let phase_col = PhaseColumn::new(sorted.iter().map(|(_, i)| i.phase.as_str()));
    let today = Local::now().date_naive();
    for (_, item) in &sorted {
        println!("{}", format_row(item, &phase_col.badge(&item.phase), today));
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
