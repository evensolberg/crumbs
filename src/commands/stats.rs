use std::path::Path;

use anyhow::Result;
use console::Style;

use crate::{item::{ItemType, Status}, store};

pub fn run(dir: &Path) -> Result<()> {
    let items = store::load_all(dir)?;

    if items.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    let total = items.len();
    let open = items.iter().filter(|(_, i)| i.status == Status::Open).count();
    let in_progress = items.iter().filter(|(_, i)| i.status == Status::InProgress).count();
    let closed = items.iter().filter(|(_, i)| i.status == Status::Closed).count();

    let bold = Style::new().bold();
    let dim = Style::new().dim();

    println!("{}", bold.apply_to("Status"));
    println!("  ○ Open:        {open}");
    println!("  {} In Progress:  {in_progress}", Style::new().yellow().apply_to("●"));
    println!("  {} Closed:       {closed}", dim.apply_to("✓"));
    println!("  {} Total:        {total}", bold.apply_to("∑"));

    println!();
    println!("{}", bold.apply_to("By Type"));
    for (type_name, style) in [
        (ItemType::Bug, Style::new().red()),
        (ItemType::Feature, Style::new().cyan()),
        (ItemType::Epic, Style::new().magenta()),
        (ItemType::Task, Style::new()),
        (ItemType::Idea, Style::new().dim()),
    ] {
        let count = items.iter().filter(|(_, i)| i.item_type == type_name).count();
        if count > 0 {
            println!("  {:<12} {count}", style.apply_to(format!("{type_name}:")));
        }
    }

    println!();
    println!("{}", bold.apply_to("By Priority"));
    for p in 0u8..=4 {
        let count = items.iter().filter(|(_, i)| i.priority == p).count();
        if count > 0 {
            let p_style = crate::color::priority(p);
            println!("  {:<12} {count}", p_style.apply_to(format!("P{p}:")));
        }
    }

    Ok(())
}
