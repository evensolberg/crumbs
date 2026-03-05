use std::path::Path;

use anyhow::Result;

use crate::store;

pub fn run(dir: &Path, format: &str, output: Option<&Path>) -> Result<()> {
    let items: Vec<_> = store::load_all(dir)?.into_iter().map(|(_, i)| i).collect();

    let content = match format {
        "json" => serde_json::to_string_pretty(&items)?,
        "toon" => serde_toon::to_string(&items)?,
        "csv" => {
            let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
            wtr.write_record([
                "id",
                "title",
                "status",
                "type",
                "priority",
                "tags",
                "created",
                "updated",
                "closed_reason",
                "dependencies",
                "due",
            ])?;
            for item in &items {
                wtr.write_record([
                    &item.id,
                    &item.title,
                    &item.status.to_string(),
                    &item.item_type.to_string(),
                    &item.priority.to_string(),
                    &item.tags.join("|"),
                    &item.created.to_string(),
                    &item.updated.to_string(),
                    &item.closed_reason,
                    &item.dependencies.join("|"),
                    &item.due.map(|d| d.to_string()).unwrap_or_default(),
                ])?;
            }
            String::from_utf8(wtr.into_inner()?)?
        }
        other => anyhow::bail!("unknown format: {other} (expected csv, json, or toon)"),
    };

    match output {
        Some(path) => std::fs::write(path, content)?,
        None => print!("{content}"),
    }

    Ok(())
}
