use std::path::Path;

use anyhow::Result;

use crate::{item::Item, store};

/// Serialize `items` to the requested format string without any I/O.
///
/// # Errors
///
/// Returns an error if serialization fails or `format` is unrecognized.
pub fn items_to_string(items: &[Item], format: &str) -> Result<String> {
    match format {
        "json" => Ok(serde_json::to_string_pretty(items)?),
        "toon" => Ok(serde_toon::to_string(&items.to_vec())?),
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
                "story_points",
            ])?;
            for item in items {
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
                    &item
                        .story_points
                        .map(|sp| sp.to_string())
                        .unwrap_or_default(),
                ])?;
            }
            Ok(String::from_utf8(wtr.into_inner()?)?)
        }
        other => anyhow::bail!("unknown format: {other} (expected csv, json, or toon)"),
    }
}

/// Load all items from `dir` and serialize them to the requested format.
///
/// # Errors
///
/// Returns an error if items cannot be loaded or serialization fails.
pub fn to_string(dir: &Path, format: &str) -> Result<String> {
    let items: Vec<_> = store::load_all(dir)?.into_iter().map(|(_, i)| i).collect();
    items_to_string(&items, format)
}

/// # Errors
///
/// Returns an error if export fails or the output file cannot be written.
pub fn run(dir: &Path, format: &str, output: Option<&Path>) -> Result<()> {
    let content = to_string(dir, format)?;
    match output {
        Some(path) => std::fs::write(path, content)?,
        None => print!("{content}"),
    }
    Ok(())
}
