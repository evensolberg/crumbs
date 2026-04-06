use chrono::NaiveDate;
use console::Style;

use crate::{color, commands::start::active_start_ts, item::Item};

/// Renders one item as a terminal row string (no trailing newline).
///
/// `phase_badge` must already be padded to the desired column width by the
/// caller (typically via [`console::measure_text_width`] + a precomputed
/// spaces string). `today` is supplied once by the caller so it is not
/// recomputed per item.
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
