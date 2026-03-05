use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;
use console::Style;

use crate::{color, store};

pub fn run(dir: &Path, id: &str) -> Result<()> {
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
                        Style::new().red().bold().apply_to(format!("{due} (overdue)"))
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
            if !item.description.is_empty() {
                println!();
                println!("{}", item.description);
                println!();
            }
            println!("  File:     {}", dim.apply_to(path.display()));
        }
    }
    Ok(())
}
