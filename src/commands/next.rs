use std::path::Path;

use anyhow::Result;

use crate::{item::Status, store};

pub fn run(dir: &Path) -> Result<()> {
    let items = store::load_all(dir)?;
    let candidate = items
        .into_iter()
        .filter(|(_, item)| item.status != Status::Closed)
        .min_by_key(|(_, item)| (item.priority, item.created));

    match candidate {
        None => println!("No open items."),
        Some((path, _)) => super::show::run(dir, &{
            let item = store::read_item(&path)?;
            item.id
        })?,
    }
    Ok(())
}
