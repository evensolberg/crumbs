use std::path::Path;

use anyhow::Result;
use console::Style;

use crate::{item::Item, store};

/// Infer the import format from the file extension, falling back to `explicit`
/// when provided.
///
/// Recognised extensions: `.json`, `.csv`.
/// Returns an error for any other extension (or no extension) when `explicit`
/// is `None`.
///
/// Note: TOON import is not supported because `serde_toon` serialises enum
/// variants as bare strings and cannot round-trip them back through
/// `from_slice`. Use `export --format json` for a portable round-trip format.
///
/// # Errors
///
/// Returns an error if the format cannot be inferred and `explicit` is `None`.
pub fn infer_format<'a>(path: &Path, explicit: Option<&'a str>) -> Result<&'a str> {
    if let Some(f) = explicit {
        return Ok(f);
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => Ok("json"),
        Some("csv") => Ok("csv"),
        Some(ext) => {
            anyhow::bail!("cannot infer import format from .{ext}; use --format json or csv")
        }
        None => anyhow::bail!("file has no extension; use --format json or csv"),
    }
}

/// Deserialise items from `bytes` using `format` (`"json"`, `"toon"`, or
/// `"csv"`).
///
/// # Errors
///
/// Returns an error if the format is unrecognised or the input is malformed.
fn parse_items(bytes: &[u8], format: &str) -> Result<Vec<Item>> {
    match format {
        "json" => Ok(serde_json::from_slice(bytes)?),
        "csv" => parse_csv(bytes),
        // TOON import is intentionally unsupported: serde_toon serialises enum
        // variants as bare strings and cannot round-trip them via from_slice.
        "toon" => anyhow::bail!(
            "TOON import is not supported; export as JSON instead (crumbs export --format json)"
        ),
        other => anyhow::bail!("unsupported import format: {other} (expected json or csv)"),
    }
}

fn parse_csv(bytes: &[u8]) -> Result<Vec<Item>> {
    use chrono::NaiveDate;

    let mut rdr = csv::Reader::from_reader(bytes);
    let headers = rdr.headers()?.clone();
    let mut items = Vec::new();

    for result in rdr.records() {
        let rec = result?;
        let col = |name: &str| -> &str {
            headers
                .iter()
                .position(|h| h == name)
                .and_then(|i| rec.get(i))
                .unwrap_or("")
        };

        let parse_date = |s: &str| -> Option<NaiveDate> {
            if s.is_empty() {
                None
            } else {
                NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
            }
        };

        let split_pipe = |s: &str| -> Vec<String> {
            if s.is_empty() {
                vec![]
            } else {
                s.split('|').map(str::to_string).collect()
            }
        };

        let item = Item {
            id: col("id").to_string(),
            title: col("title").to_string(),
            status: col("status").parse().unwrap_or(crate::item::Status::Open),
            phase: col("phase").to_string(),
            item_type: col("type").parse().unwrap_or_default(),
            priority: col("priority").parse().unwrap_or(2),
            tags: split_pipe(col("tags")),
            created: parse_date(col("created"))
                .unwrap_or_else(|| chrono::Local::now().date_naive()),
            updated: parse_date(col("updated"))
                .unwrap_or_else(|| chrono::Local::now().date_naive()),
            closed_reason: col("closed_reason").to_string(),
            dependencies: split_pipe(col("dependencies")),
            blocks: split_pipe(col("blocks")),
            blocked_by: split_pipe(col("blocked_by")),
            due: parse_date(col("due")),
            story_points: {
                let s = col("story_points");
                if s.is_empty() { None } else { s.parse().ok() }
            },
            description: String::new(),
            resolution: col("resolution").to_string(),
        };
        items.push(item);
    }
    Ok(items)
}

/// Import items from a file into `dir`.
///
/// The format is inferred from the file extension unless `format` is supplied.
/// Recognised extensions: `.json`, `.toon`, `.csv`.
///
/// Fails immediately if any imported item's ID already exists in the store.
/// On success, the store is reindexed once.
///
/// # Errors
///
/// Returns an error if the format cannot be inferred, the file cannot be read,
/// the input is malformed, or any imported ID conflicts with an existing item.
pub fn run(dir: &Path, path: &Path, format: Option<&str>) -> Result<()> {
    let fmt = infer_format(path, format)?;
    let bytes = std::fs::read(path)?;
    let items = parse_items(&bytes, fmt)?;

    if items.is_empty() {
        return Ok(());
    }

    // Check for duplicate IDs within the import file itself.
    let mut seen_in_file: std::collections::HashSet<String> = std::collections::HashSet::new();
    for item in &items {
        let key = item.id.to_lowercase();
        if !seen_in_file.insert(key) {
            anyhow::bail!("ID {} appears more than once in the import file", item.id);
        }
    }

    // Check for conflicts against items already in the store.
    // unwrap_or_default: treat an unreadable/empty store as no existing items.
    let existing: std::collections::HashSet<String> = store::load_all(dir)
        .unwrap_or_default()
        .into_iter()
        .map(|(_, i)| i.id.to_lowercase())
        .collect();

    for item in &items {
        if existing.contains(&item.id.to_lowercase()) {
            anyhow::bail!(
                "ID {} already exists in the store; resolve the conflict before importing",
                item.id
            );
        }
    }

    let bold = Style::new().bold();
    for item in items {
        let path = store::write_item(dir, &item)?;
        println!("Imported {} — {}", bold.apply_to(&item.id), item.title);
        println!("  {}", path.display());
    }

    store::reindex(dir)?;
    Ok(())
}
