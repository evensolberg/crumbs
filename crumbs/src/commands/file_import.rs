use std::path::Path;

use anyhow::Result;
use console::Style;
use slugify::slugify;

use crate::{item::Item, store};

/// Return `true` if `id` matches the expected `{prefix}-{3-char alphanumeric}` format
/// using only lowercase ASCII letters and digits.
///
/// This guards against path traversal and case-insensitive collisions: imported IDs
/// are used to derive file names inside the store directory, so we reject anything
/// containing `/`, `\`, `..`, uppercase letters, or characters outside `[a-z0-9-]`.
fn is_valid_id(id: &str) -> bool {
    let mut parts = id.splitn(2, '-');
    let Some(prefix) = parts.next() else {
        return false;
    };
    let Some(suffix) = parts.next() else {
        return false;
    };
    !prefix.is_empty()
        && prefix
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && suffix.len() == 3
        && suffix
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

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
        Some("toon") => anyhow::bail!(
            "TOON import is not supported; export as JSON instead (crumbs export --format json)"
        ),
        Some(ext) => {
            anyhow::bail!("cannot infer import format from .{ext}; use --format json or csv")
        }
        None => anyhow::bail!("file has no extension; use --format json or csv"),
    }
}

/// Deserialise items from `bytes` using `format` (`"json"` or `"csv"`).
/// Passing `"toon"` returns an explicit unsupported error with a helpful message.
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
    use std::collections::HashMap;

    let mut rdr = csv::Reader::from_reader(bytes);
    let header_index: HashMap<String, usize> = rdr
        .headers()?
        .iter()
        .enumerate()
        .map(|(i, h)| (h.to_owned(), i))
        .collect();
    let mut items = Vec::new();

    for result in rdr.records() {
        let rec = result?;
        let col = |name: &str| -> &str {
            header_index
                .get(name)
                .and_then(|&i| rec.get(i))
                .unwrap_or_default()
        };

        let split_pipe = |s: &str| -> Vec<String> {
            if s.is_empty() {
                vec![]
            } else {
                s.split('|').map(str::to_string).collect()
            }
        };

        let id = col("id").to_string();
        anyhow::ensure!(!id.is_empty(), "CSV row has an empty id field");

        let parse_date = |s: &str| -> Result<Option<NaiveDate>> {
            if s.is_empty() {
                Ok(None)
            } else {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(Some)
                    .map_err(|_| {
                        anyhow::anyhow!("invalid date {s:?} for item {id:?} (expected YYYY-MM-DD)")
                    })
            }
        };
        let title = col("title").to_string();
        anyhow::ensure!(
            !title.is_empty(),
            "CSV row has an empty title field (id: {id})"
        );

        let status: crate::item::Status = {
            let s = col("status");
            if s.is_empty() {
                crate::item::Status::Open
            } else {
                s.parse()
                    .map_err(|_| anyhow::anyhow!("invalid status value {s:?} for item {id:?}"))?
            }
        };
        let item_type: crate::item::ItemType = {
            let s = col("type");
            if s.is_empty() {
                crate::item::ItemType::default()
            } else {
                s.parse()
                    .map_err(|_| anyhow::anyhow!("invalid type value {s:?} for item {id:?}"))?
            }
        };
        let priority: u8 = {
            let s = col("priority");
            if s.is_empty() {
                2
            } else {
                s.parse()
                    .map_err(|_| anyhow::anyhow!("invalid priority value {s:?} for item {id:?}"))?
            }
        };
        let story_points: Option<u8> = {
            let s = col("story_points");
            if s.is_empty() {
                None
            } else {
                Some(s.parse::<u8>().map_err(|_| {
                    anyhow::anyhow!("invalid story_points value {s:?} for item {id:?}")
                })?)
            }
        };
        let created =
            parse_date(col("created"))?.unwrap_or_else(|| chrono::Local::now().date_naive());
        let updated =
            parse_date(col("updated"))?.unwrap_or_else(|| chrono::Local::now().date_naive());
        let due = parse_date(col("due"))?;
        let item = Item {
            id,
            title,
            status,
            phase: col("phase").to_string(),
            item_type,
            priority,
            tags: split_pipe(col("tags")),
            created,
            updated,
            closed_reason: col("closed_reason").to_string(),
            dependencies: split_pipe(col("dependencies")),
            blocks: split_pipe(col("blocks")),
            blocked_by: split_pipe(col("blocked_by")),
            due,
            story_points,
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
/// Recognised extensions: `.json`, `.csv`. TOON import is not supported;
/// use `crumbs export --format json` for portable round-trips.
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

    // Validate all items before touching disk.
    let mut seen_in_file: std::collections::HashSet<String> = std::collections::HashSet::new();
    for item in &items {
        anyhow::ensure!(
            !item.title.trim().is_empty() && !slugify!(&item.title, max_length = 60).is_empty(),
            "item {:?} title {:?} must contain at least one alphanumeric character",
            item.id,
            item.title
        );
        // Enforce the expected ID format to prevent path traversal.
        // Valid form: one or more lowercase alphanumeric chars, a hyphen,
        // then exactly 3 lowercase alphanumeric chars (e.g. "cr-x7q").
        if !is_valid_id(&item.id) {
            anyhow::bail!(
                "invalid ID {:?}: must match <prefix>-<3 alphanumeric chars> (e.g. cr-x7q)",
                item.id
            );
        }
        let key = item.id.to_lowercase();
        if !seen_in_file.insert(key) {
            anyhow::bail!("ID {} appears more than once in the import file", item.id);
        }
    }

    // Validate Fibonacci story_points before writing anything.
    for item in &items {
        if let Some(sp) = item.story_points
            && !crate::item::is_fibonacci(sp)
        {
            anyhow::bail!(
                "story_points must be a Fibonacci number (1, 2, 3, 5, 8, 13, 21); got {sp} for item {:?}",
                item.id
            );
        }
    }

    // Check for conflicts against items already in the store.
    let existing: std::collections::HashSet<String> = store::load_all(dir)?
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
