use console::Style;

use crate::item::{ItemType, Status};

#[must_use]
pub const fn priority(p: u8) -> Style {
    match p {
        0 => Style::new().red().bold(),
        1 => Style::new().red(),
        2 => Style::new().yellow(),
        3 => Style::new(),
        _ => Style::new().dim(),
    }
}

#[must_use]
pub const fn item_type(t: &ItemType) -> Style {
    match t {
        ItemType::Bug => Style::new().red(),
        ItemType::Feature => Style::new().cyan(),
        ItemType::Epic => Style::new().magenta(),
        ItemType::Idea => Style::new().dim(),
        ItemType::Task => Style::new(),
    }
}

#[must_use]
pub const fn status_icon(s: &Status) -> &'static str {
    match s {
        Status::Open => "○",
        Status::InProgress => "●",
        Status::Blocked => "⊘",
        Status::Deferred => "◷",
        Status::Closed => "✓",
    }
}

#[must_use]
pub fn status_icon_styled(s: &Status) -> String {
    let icon = status_icon(s);
    match s {
        Status::Open => icon.to_string(),
        Status::InProgress => Style::new().yellow().apply_to(icon).to_string(),
        Status::Blocked => Style::new().red().apply_to(icon).to_string(),
        Status::Deferred | Status::Closed => Style::new().dim().apply_to(icon).to_string(),
    }
}
