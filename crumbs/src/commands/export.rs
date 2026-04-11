use std::path::Path;

use anyhow::Result;

use crate::{item::Item, store};

/// Render `items` as a flat or grouped Markdown document.
///
/// `group` is an optional field name to group by: `"type"`, `"priority"`,
/// `"phase"`, or `"status"`. Items in each group are rendered as a markdown
/// table under a `## <group-value>` heading.  Items without a group value
/// (empty phase, etc.) go under `## Uncategorized`.
fn items_to_markdown(items: &[Item], group: Option<&str>) -> Result<String> {
    let header = "| ID | Title | Status | Type | P | Phase | Tags |\n\
                  |:---|:------|:-------|:-----|:-:|:------|:-----|\n";
    let esc = |s: &str| s.replace('|', "\\|");
    let row = |item: &Item| {
        format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            item.id,
            esc(&item.title),
            item.status,
            item.item_type,
            item.priority,
            esc(&item.phase),
            esc(&item.tags.join(", ")),
        )
    };

    let Some(field) = group else {
        let mut out = String::from(header);
        for item in items {
            out.push_str(&row(item));
        }
        return Ok(out);
    };

    // Validate field before building output.
    match field {
        "type" | "priority" | "phase" | "status" => {}
        other => anyhow::bail!(
            "unknown group field: {other} (expected type, priority, phase, or status)"
        ),
    }

    let key_of = |item: &Item| -> String {
        match field {
            "type" => {
                let s = item.item_type.to_string();
                let mut c = s.chars();
                c.next().map_or_else(String::new, |f| {
                    f.to_uppercase().collect::<String>() + c.as_str()
                })
            }
            "priority" => format!("P{}", item.priority),
            "phase" if item.phase.is_empty() => "Uncategorized".to_string(),
            "phase" => item.phase.clone(),
            "status" => item.status.to_string(),
            _ => unreachable!("field validated above"),
        }
    };

    // Group preserving insertion order.
    let mut order: Vec<String> = Vec::new();
    let mut groups: std::collections::HashMap<String, Vec<&Item>> =
        std::collections::HashMap::new();
    for item in items {
        let k = key_of(item);
        if !groups.contains_key(&k) {
            order.push(k.clone());
        }
        groups.entry(k).or_default().push(item);
    }

    let mut out = String::new();
    for key in &order {
        out.push_str("## ");
        out.push_str(key);
        out.push_str("\n\n");
        out.push_str(header);
        for item in &groups[key] {
            out.push_str(&row(item));
        }
        out.push('\n');
    }
    Ok(out)
}

/// Serialize `items` to the requested format string without any I/O.
///
/// Supported formats: `csv`, `json`, `toon`, `markdown`, `markdown?group=<field>`.
///
/// # Errors
///
/// Returns an error if serialization fails or `format` is unrecognized.
pub fn items_to_string(items: &[Item], format: &str) -> Result<String> {
    // Handle `markdown` and `markdown?group=<field>`.
    if format == "markdown" {
        return items_to_markdown(items, None);
    }
    if let Some(rest) = format.strip_prefix("markdown?group=") {
        return items_to_markdown(items, Some(rest));
    }

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
                "blocks",
                "blocked_by",
                "due",
                "story_points",
                "resolution",
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
                    &item.blocks.join("|"),
                    &item.blocked_by.join("|"),
                    &item.due.map(|d| d.to_string()).unwrap_or_default(),
                    &item
                        .story_points
                        .map(|sp| sp.to_string())
                        .unwrap_or_default(),
                    &item.resolution,
                ])?;
            }
            Ok(String::from_utf8(wtr.into_inner()?)?)
        }
        other => anyhow::bail!("unknown format: {other} (expected csv, json, markdown, or toon)"),
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
            dependencies: vec![],
            blocks: vec!["cr-aaa".to_string(), "cr-bbb".to_string()],
            blocked_by: vec!["cr-zzz".to_string()],
            due: None,
            description: String::new(),
            story_points: None,
            phase: String::new(),
            resolution: String::new(),
        }
    }

    fn item_with(id: &str, title: &str, item_type: ItemType, priority: u8, phase: &str) -> Item {
        Item {
            id: id.to_string(),
            title: title.to_string(),
            item_type,
            priority,
            phase: phase.to_string(),
            ..sample_item()
        }
    }

    #[test]
    fn export_markdown_flat_contains_header_and_row() {
        let items = vec![sample_item()];
        let md = items_to_string(&items, "markdown").unwrap();
        assert!(md.contains("| ID |"), "missing table header");
        assert!(md.contains("cr-t01"), "missing item id");
        assert!(md.contains("Test"), "missing item title");
    }

    #[test]
    fn export_markdown_grouped_by_type_has_sections() {
        let items = vec![
            item_with("cr-a01", "A Feature", ItemType::Feature, 2, ""),
            item_with("cr-b01", "A Bug", ItemType::Bug, 1, ""),
        ];
        let md = items_to_string(&items, "markdown?group=type").unwrap();
        assert!(md.contains("## Feature"), "missing Feature section");
        assert!(md.contains("## Bug"), "missing Bug section");
        assert!(md.contains("cr-a01"), "missing feature item");
        assert!(md.contains("cr-b01"), "missing bug item");
    }

    #[test]
    fn export_markdown_grouped_by_priority_has_sections() {
        let items = vec![
            item_with("cr-p0", "Critical", ItemType::Task, 0, ""),
            item_with("cr-p2", "Normal", ItemType::Task, 2, ""),
        ];
        let md = items_to_string(&items, "markdown?group=priority").unwrap();
        assert!(md.contains("## P0"), "missing P0 section");
        assert!(md.contains("## P2"), "missing P2 section");
    }

    #[test]
    fn export_markdown_grouped_by_phase_uses_uncategorized_for_empty() {
        let items = vec![
            item_with("cr-ph1", "Has Phase", ItemType::Task, 2, "alpha"),
            item_with("cr-ph2", "No Phase", ItemType::Task, 2, ""),
        ];
        let md = items_to_string(&items, "markdown?group=phase").unwrap();
        assert!(md.contains("## alpha"), "missing alpha section");
        assert!(
            md.contains("## Uncategorized"),
            "missing Uncategorized section"
        );
        assert!(md.contains("cr-ph1"), "missing phased item");
        assert!(md.contains("cr-ph2"), "missing unphased item");
    }

    #[test]
    fn export_markdown_unknown_group_field_errors() {
        let items = vec![sample_item()];
        let result = items_to_string(&items, "markdown?group=banana");
        assert!(result.is_err(), "expected error for unknown group field");
    }

    #[test]
    fn export_csv_includes_blocks_and_blocked_by() {
        let csv = items_to_string(&[sample_item()], "csv").unwrap();
        let mut rdr = csv::Reader::from_reader(csv.as_bytes());
        let headers = rdr.headers().unwrap().clone();
        let cols: Vec<&str> = headers.iter().collect();

        assert!(
            !cols.contains(&"dependencies"),
            "dependencies column should be removed"
        );
        assert!(cols.contains(&"blocks"), "blocks column should exist");
        assert!(
            cols.contains(&"blocked_by"),
            "blocked_by column should exist"
        );

        let row = rdr.records().next().unwrap().unwrap();
        let col = |name: &str| cols.iter().position(|c| *c == name).unwrap();
        assert_eq!(row.get(col("blocks")), Some("cr-aaa|cr-bbb"));
        assert_eq!(row.get(col("blocked_by")), Some("cr-zzz"));
    }
}
