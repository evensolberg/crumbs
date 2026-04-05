use std::path::Path;

use anyhow::{Result, bail};
use chrono::{Local, NaiveDateTime};
use console::Style;

use crate::{color, commands::start::active_start_ts, store};

/// Sum elapsed seconds across all matched `[start]`/`[stop]` pairs in a description.
fn total_tracked_secs(description: &str) -> i64 {
    let mut total: i64 = 0;
    let mut current_start: Option<NaiveDateTime> = None;
    for line in description.lines() {
        let t = line.trim();
        if t.starts_with("[start]") {
            let rest = t.trim_start_matches("[start]").trim();
            if let Ok(dt) =
                NaiveDateTime::parse_from_str(&rest[..rest.len().min(19)], "%Y-%m-%d %H:%M:%S")
            {
                current_start = Some(dt);
            }
        } else if t.starts_with("[stop]")
            && let Some(start_dt) = current_start.take()
        {
            let rest = t.trim_start_matches("[stop]").trim();
            if let Ok(stop_dt) =
                NaiveDateTime::parse_from_str(&rest[..rest.len().min(19)], "%Y-%m-%d %H:%M:%S")
            {
                total += (stop_dt - start_dt).num_seconds().max(0);
            }
        }
    }
    total
}

fn show_one(dir: &Path, id: &str) -> Result<()> {
    match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some((path, item)) => {
            let p_style = color::priority(item.priority);
            let t_style = color::item_type(&item.item_type);
            let dim = Style::new().dim();

            println!(
                "{} — {}",
                Style::new().bold().apply_to(&item.id),
                item.title
            );
            println!(
                "  Status:   {}",
                color::status_icon_styled(&item.status) + " " + &item.status.to_string()
            );
            println!("  Type:     {}", t_style.apply_to(&item.item_type));
            println!(
                "  Priority: {}",
                p_style.apply_to(format!("P{}", item.priority))
            );
            if let Some(due) = item.due {
                let today = Local::now().date_naive();
                if due < today {
                    println!(
                        "  Due:      {}",
                        Style::new()
                            .red()
                            .bold()
                            .apply_to(format!("{due} (overdue)"))
                    );
                } else {
                    println!("  Due:      {due}");
                }
            }
            if !item.tags.is_empty() {
                println!("  Tags:     {}", item.tags.join(", "));
            }
            println!("  Created:  {}", dim.apply_to(item.created));
            println!("  Updated:  {}", dim.apply_to(item.updated));
            if !item.closed_reason.is_empty() {
                println!("  Closed:   {}", item.closed_reason);
            }
            if !item.dependencies.is_empty() {
                println!("  Deps:     {}", item.dependencies.join(", "));
            }
            if !item.blocks.is_empty() {
                println!("  Blocks:   {}", item.blocks.join(", "));
            }
            if !item.blocked_by.is_empty() {
                println!(
                    "  Blocked:  {}",
                    Style::new().red().apply_to(item.blocked_by.join(", "))
                );
            }
            if let Some(sp) = item.story_points {
                println!("  Points:   {sp}");
            }
            if let Some(ref p) = item.phase {
                println!("  Phase:    {p}");
            }
            if !item.description.is_empty() {
                println!();
                println!("{}", item.description);
                let active_ts = active_start_ts(&item.description);
                let live_secs = active_ts
                    .as_deref()
                    .and_then(|ts| NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S").ok())
                    .map_or(0, |start| {
                        Local::now()
                            .naive_local()
                            .signed_duration_since(start)
                            .num_seconds()
                            .max(0)
                    });
                let tracked = total_tracked_secs(&item.description) + live_secs;
                if tracked > 0 {
                    let running_marker = if active_ts.is_some() {
                        "  ▶ running"
                    } else {
                        ""
                    };
                    println!();
                    println!(
                        "  Tracked:  {}{}",
                        dim.apply_to(super::stop::format_elapsed(tracked)),
                        running_marker
                    );
                }
                println!();
            }
            println!("  File:     {}", dim.apply_to(path.display()));
        }
    }
    Ok(())
}

/// # Errors
///
/// Returns an error if any item cannot be found or the store cannot be read.
pub fn run(dir: &Path, ids: &[String]) -> Result<()> {
    for (i, id) in ids.iter().enumerate() {
        if i > 0 {
            println!();
        }
        show_one(dir, id)?;
    }
    Ok(())
}
