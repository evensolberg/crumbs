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
                "phase",
                "type",
                "priority",
                "tags",
                "created",
                "updated",
                "closed_reason",
                "dependencies",
                "blocks",
                "blocked_by",
                "due",
                "story_points",
            ])?;
            for item in items {
                wtr.write_record([
                    &item.id,
                    &item.title,
                    &item.status.to_string(),
                    &item.phase,
                    &item.item_type.to_string(),
                    &item.priority.to_string(),
                    &item.tags.join("|"),
                    &item.created.to_string(),
                    &item.updated.to_string(),
                    &item.closed_reason,
                    &item.dependencies.join("|"),
                    &item.blocks.join("|"),
                    &item.blocked_by.join("|"),
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::items_to_string;
    use crate::item::{Item, ItemType, Status};

    fn sample_item() -> Item {
        Item {
            id: "cr-t01".to_string(),
            title: "Test".to_string(),
            status: Status::Open,
            item_type: ItemType::Task,
            priority: 2,
            tags: vec![],
            created: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            closed_reason: String::new(),
            dependencies: vec!["cr-dep".to_string()],
            blocks: vec!["cr-aaa".to_string(), "cr-bbb".to_string()],
            blocked_by: vec!["cr-zzz".to_string()],
            due: None,
            description: String::new(),
            story_points: None,
            phase: String::new(),
        }
    }

    #[test]
    fn export_csv_includes_blocks_and_blocked_by() {
        let csv = items_to_string(&[sample_item()], "csv").unwrap();
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let headers = rdr.headers().unwrap().clone();
        let cols: Vec<&str> = headers.iter().collect();

        let dep_idx = cols.iter().position(|c| *c == "dependencies").unwrap();
        assert_eq!(cols.get(dep_idx + 1), Some(&"blocks"));
        assert_eq!(cols.get(dep_idx + 2), Some(&"blocked_by"));

        let row = rdr.records().next().unwrap().unwrap();
        let col = |name: &str| cols.iter().position(|c| *c == name).unwrap();
        assert_eq!(row.get(col("blocks")), Some("cr-aaa|cr-bbb"));
        assert_eq!(row.get(col("blocked_by")), Some("cr-zzz"));
    }
}
