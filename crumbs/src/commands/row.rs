use std::collections::HashMap;

use chrono::NaiveDate;
use console::{Style, measure_text_width};

use crate::{color, commands::start::active_start_ts, item::Item};

/// Precomputed column alignment for phase badges.
///
/// Build once from the set of items to display, then call [`PhaseColumn::badge`]
/// per item to get a fixed-width `[phase]` string.
pub struct PhaseColumn {
    max_width: usize,
    spaces: String,
    widths: HashMap<String, usize>,
}

impl PhaseColumn {
    /// Measure every phase string once and record the widest one.
    pub fn new<'a>(phases: impl Iterator<Item = &'a str>) -> Self {
        let widths: HashMap<String, usize> = phases
            .map(|p| (p.to_owned(), measure_text_width(p)))
            .collect();
        let max_width = widths.values().copied().max().unwrap_or(0);
        let spaces = " ".repeat(max_width);
        Self {
            max_width,
            spaces,
            widths,
        }
    }

    /// Render a phase value as a bracket-wrapped, right-padded badge.
    #[must_use]
    pub fn badge(&self, phase: &str) -> String {
        let w = self
            .widths
            .get(phase)
            .copied()
            .unwrap_or_else(|| measure_text_width(phase));
        let padding = self.max_width.saturating_sub(w);
        format!("[{}{}]", phase, &self.spaces[..padding])
    }
}

/// Renders one item as a terminal row string (no trailing newline).
///
/// `phase_badge` must already be padded to the desired column width
/// (see [`PhaseColumn::badge`]). `today` is supplied once by the caller
/// so it is not recomputed per item.
#[must_use]
pub fn format_row(item: &Item, phase_badge: &str, today: NaiveDate) -> String {
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
    format!(
        "{icon} {} {} {} {} {}{timer_marker}{tags}{due_marker}{points_marker}",
        item.id,
        p_style.apply_to(format!("[P{}]", item.priority)),
        phase_badge,
        t_style.apply_to(format!("[{}]", item.item_type)),
        item.title,
    )
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::format_row;
    use crate::item::{Item, ItemType, Status};

    fn base_item() -> Item {
        Item {
            id: "cr-abc".to_string(),
            title: "Test Item".to_string(),
            status: Status::Open,
            item_type: ItemType::Task,
            priority: 2,
            tags: vec![],
            created: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            closed_reason: String::new(),
            dependencies: vec![],
            blocks: vec![],
            blocked_by: vec![],
            due: None,
            description: String::new(),
            story_points: None,
            phase: String::new(),
        }
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 4, 6).unwrap()
    }

    #[test]
    fn format_row_contains_core_fields() {
        let item = base_item();
        let row = format_row(&item, "[   ]", today());
        assert!(row.contains("cr-abc"), "missing id");
        assert!(row.contains("[P2]"), "missing priority badge");
        assert!(row.contains("[   ]"), "missing phase badge");
        assert!(row.contains("Test Item"), "missing title");
    }

    #[test]
    fn format_row_shows_tags() {
        let item = Item {
            tags: vec!["foo".to_string(), "bar".to_string()],
            ..base_item()
        };
        let row = format_row(&item, "[]", today());
        assert!(row.contains("[foo, bar]"), "missing tags, got:\n{row}");
    }

    #[test]
    fn format_row_shows_overdue_marker() {
        let item = Item {
            due: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            ..base_item()
        };
        let row = format_row(&item, "[]", today());
        assert!(row.contains("!due"), "missing overdue marker, got:\n{row}");
    }

    #[test]
    fn format_row_shows_story_points() {
        let item = Item {
            story_points: Some(5),
            ..base_item()
        };
        let row = format_row(&item, "[]", today());
        assert!(row.contains("[5sp]"), "missing story points, got:\n{row}");
    }

    #[test]
    fn format_row_shows_future_due_date() {
        let item = Item {
            due: Some(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()),
            ..base_item()
        };
        let row = format_row(&item, "[]", today());
        assert!(
            row.contains("due:2027-01-01"),
            "missing future due date, got:\n{row}"
        );
    }

    #[test]
    fn format_row_shows_timer_marker() {
        let item = Item {
            description: "[start] 2026-04-06 10:00:00".to_string(),
            ..base_item()
        };
        let row = format_row(&item, "[]", today());
        assert!(row.contains('▶'), "missing timer marker, got:\n{row}");
    }
}
