use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use slugify::slugify;

use crate::item::Item;

/// Atomically overwrite `path` with `content`.
///
/// Writes to a sibling temp file then renames, so a mid-write crash or kill
/// leaves the original file intact.
///
/// # Errors
///
/// Returns an error if the temp file cannot be created, written, or renamed.
pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let dir = path.parent().context("item path has no parent directory")?;
    let mut tmp = tempfile::NamedTempFile::new_in(dir).context("create temp file")?;
    tmp.write_all(content.as_bytes())
        .context("write temp file")?;
    tmp.flush().context("flush temp file")?;
    tmp.persist(path)
        .map_err(|e| e.error)
        .context("rename temp file")?;
    Ok(())
}

/// Rewrite the YAML frontmatter of an existing item file, preserving the body verbatim.
///
/// # Errors
///
/// Returns an error if the item cannot be serialized or the file cannot be written.
pub fn rewrite_frontmatter(path: &Path, item: &Item) -> Result<()> {
    let mut fm = item.clone();
    fm.description.clear();
    let frontmatter = serde_yaml_ng::to_string(&fm)?;
    let raw = std::fs::read_to_string(path)?;
    let body = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, b)| b))
        .unwrap_or("");
    atomic_write(path, &format!("---\n{frontmatter}---\n{body}"))
}

#[must_use]
pub fn item_path(dir: &Path, item: &Item) -> PathBuf {
    let slug = slugify!(&item.title, max_length = 60);
    dir.join(format!("{slug}.md"))
}

/// Build the fallback path (slug + full ID) used when the base slug collides.
///
/// Using the full ID (prefix + suffix) rather than just the suffix ensures
/// uniqueness even when items from different prefixes share the same 3-char
/// suffix and the same title slug.
fn fallback_path(dir: &Path, item: &Item) -> PathBuf {
    let slug = slugify!(&item.title, max_length = 60);
    dir.join(format!("{slug}-{}.md", item.id))
}

/// # Errors
///
/// Returns an error if the item cannot be serialized or the file cannot be created.
pub fn write_item(dir: &Path, item: &Item) -> Result<PathBuf> {
    // description lives in the markdown body, not the frontmatter.
    // Clear it before YAML serialization so it never leaks into the front matter.
    let mut fm = item.clone();
    fm.description = String::new();
    let frontmatter = serde_yaml_ng::to_string(&fm).context("serialize frontmatter")?;
    let body = if item.description.is_empty() {
        format!("# {}\n", item.title)
    } else {
        format!("# {}\n\n{}\n", item.title, item.description.trim())
    };
    let content = format!("---\n{frontmatter}---\n\n{body}");

    // Use create_new(true) for an atomic exclusive create, avoiding a TOCTOU
    // race between an existence check and the write.  On collision fall back to
    // the ID-suffixed path.
    let base = item_path(dir, item);
    let path = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&base)
    {
        Ok(mut f) => {
            f.write_all(content.as_bytes()).context("write item file")?;
            base
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let fallback = fallback_path(dir, item);
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&fallback)
                .context("write item file (fallback path)")?
                .write_all(content.as_bytes())
                .context("write item file (fallback path)")?;
            fallback
        }
        Err(e) => return Err(e).context("write item file"),
    };
    Ok(path)
}

/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn read_item(path: &Path) -> Result<Item> {
    let raw = std::fs::read_to_string(path).context("read item file")?;
    parse_item(&raw).context("parse item")
}

/// # Errors
///
/// Returns an error if the raw string lacks frontmatter delimiters or contains invalid YAML.
pub fn parse_item(raw: &str) -> Result<Item> {
    let (fm, body) = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n"))
        .ok_or_else(|| anyhow!("missing frontmatter delimiters"))?;
    let mut item: Item = serde_yaml_ng::from_str(fm).context("deserialize frontmatter")?;
    // The body must start with `# <title>` (possibly preceded by a blank line).
    // We verify the heading matches and extract everything after it as the
    // description, so hand-edits that remove the heading are detected early.
    let body_trimmed = body.trim_start_matches('\n');
    let (heading_line, rest) = body_trimmed.split_once('\n').unwrap_or((body_trimmed, ""));
    let expected_heading = format!("# {}", item.title);
    if heading_line != expected_heading {
        eprintln!(
            "warning: body heading {:?} does not match title {:?}; description may be incomplete",
            heading_line, item.title
        );
    }
    let description = rest.trim_matches('\n').to_string();
    if !description.is_empty() {
        item.description = description;
    }
    Ok(item)
}

/// Promote a legacy `depends` list to bidirectional `blocked_by`/`blocks`
/// links and rewrite both sides to disk.
///
/// Called by [`load_all`] for any item that still carries a non-empty
/// `dependencies` vec. After this function returns the item's
/// `dependencies` is empty and `blocked_by` is extended; each referenced
/// item's `blocks` list is extended and its file is rewritten atomically.
///
/// Unknown dependency IDs are silently ignored so that cross-store or
/// deleted references do not block migration.
///
/// # Errors
///
/// Returns an error if reading or rewriting any item file fails.
fn migrate_depends(path: &Path, item: &mut Item, all: &[(PathBuf, Item)]) -> Result<()> {
    let ids = std::mem::take(&mut item.dependencies);
    for dep_id in &ids {
        if !item.blocked_by.contains(dep_id) {
            item.blocked_by.push(dep_id.clone());
        }
        if let Some((dep_path, _)) = all.iter().find(|(_, i)| i.id.eq_ignore_ascii_case(dep_id)) {
            // Read fresh from disk — a previous migration call in this same
            // load_all pass may have already updated this file.
            let mut dep = read_item(dep_path)?;
            if !dep.blocks.contains(&item.id) {
                dep.blocks.push(item.id.clone());
                rewrite_frontmatter(dep_path, &dep)?;
            }
        }
    }
    rewrite_frontmatter(path, item)?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the directory cannot be read.
pub fn load_all(dir: &Path) -> Result<Vec<(PathBuf, Item)>> {
    let mut items = Vec::new();
    let mut skipped = 0usize;
    for entry in std::fs::read_dir(dir).context("read dir")? {
        let entry: std::fs::DirEntry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e: &std::ffi::OsStr| e.to_str()) == Some("md") {
            match read_item(&path) {
                Ok(item) => items.push((path, item)),
                Err(e) => {
                    eprintln!("warning: skipping {}: {e}", path.display());
                    skipped += 1;
                }
            }
        }
    }
    if skipped > 0 {
        eprintln!("warning: {skipped} item(s) skipped due to parse errors");
    }
    // Lazy one-time migration: promote legacy `dependencies` to blocked_by/blocks.
    let to_migrate: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, (_, item))| !item.dependencies.is_empty())
        .map(|(i, _)| i)
        .collect();
    if !to_migrate.is_empty() {
        let snapshot: Vec<(PathBuf, Item)> = items.clone();
        for idx in to_migrate {
            let (path, item) = &mut items[idx];
            if let Err(e) = migrate_depends(path, item, &snapshot) {
                eprintln!(
                    "warning: depends migration failed for {}: {e}",
                    path.display()
                );
            }
        }
        // Reload all items from disk so that both sides of the migration are
        // reflected in the returned vec (blocker.blocks updated on disk above).
        items.clear();
        for entry in std::fs::read_dir(dir).context("read dir (post-migration)")? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                match read_item(&path) {
                    Ok(item) => items.push((path, item)),
                    Err(e) => eprintln!("warning: skipping {}: {e}", path.display()),
                }
            }
        }
    }
    items.sort_by(|a, b| a.1.id.cmp(&b.1.id));
    Ok(items)
}

/// # Errors
///
/// Returns an error if items cannot be loaded or the CSV index cannot be written.
pub fn reindex(dir: &Path) -> Result<()> {
    let items = load_all(dir)?;
    let index_path = dir.join("index.csv");
    let mut wtr = csv::Writer::from_path(&index_path).context("open index.csv")?;
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
    for (_, item) in &items {
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
    wtr.flush()?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the store cannot be read.
pub fn find_by_id(dir: &Path, id: &str) -> Result<Option<(PathBuf, Item)>> {
    let items = load_all(dir)?;
    let id_lower = id.to_lowercase();
    if let Some(found) = items
        .iter()
        .find(|(_, item)| item.id.to_lowercase() == id_lower)
    {
        return Ok(Some(found.clone()));
    }
    // If the input looks like a bare suffix (no '-'), try prepending the store prefix.
    if !id.contains('-') {
        let prefix = crate::store_config::load(dir).prefix;
        let full_id = format!("{prefix}-{id_lower}");
        return Ok(items
            .into_iter()
            .find(|(_, item)| item.id.to_lowercase() == full_id));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use tempfile::tempdir;

    use super::*;
    use crate::item::{Item, ItemType, Status};

    fn sample_item(id: &str, title: &str) -> Item {
        Item {
            id: id.to_string(),
            title: title.to_string(),
            status: Status::Open,
            item_type: ItemType::Task,
            priority: 2,
            tags: vec!["project/test".to_string()],
            created: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            closed_reason: String::new(),
            dependencies: Vec::new(),
            blocks: Vec::new(),
            blocked_by: Vec::new(),
            due: None,
            description: String::new(),
            story_points: None,
            phase: String::new(),
            resolution: String::new(),
        }
    }

    // --- parse_item ---

    #[test]
    fn parse_item_valid() {
        let raw = "---\nid: bc-abc\ntitle: Hello\nstatus: open\ntype: task\npriority: 2\ntags:\n  - foo\ncreated: 2026-03-01\nupdated: 2026-03-01\nclosed_reason: ''\ndependencies: []\n---\n\n# Hello\n";
        let item = parse_item(raw).unwrap();
        assert_eq!(item.id, "bc-abc");
        assert_eq!(item.title, "Hello");
        assert_eq!(item.status, Status::Open);
    }

    #[test]
    fn parse_item_missing_delimiters() {
        let raw = "id: bc-abc\ntitle: Hello\n";
        assert!(parse_item(raw).is_err());
    }

    #[test]
    fn parse_item_bad_yaml() {
        let raw = "---\n: : :\n---\n";
        assert!(parse_item(raw).is_err());
    }

    // --- write_item / read_item round-trip ---

    #[test]
    fn write_then_read_round_trip() {
        let dir = tempdir().unwrap();
        let item = sample_item("bc-xyz", "Round Trip Test");
        let path = write_item(dir.path(), &item).unwrap();
        let loaded = read_item(&path).unwrap();
        assert_eq!(loaded.id, item.id);
        assert_eq!(loaded.title, item.title);
        assert_eq!(loaded.status, item.status);
        assert_eq!(loaded.item_type, item.item_type);
        assert_eq!(loaded.priority, item.priority);
        assert_eq!(loaded.tags, item.tags);
    }

    #[test]
    fn write_item_creates_file_with_heading() {
        let dir = tempdir().unwrap();
        let item = sample_item("bc-h1t", "My Heading");
        let path = write_item(dir.path(), &item).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# My Heading"));
    }

    #[test]
    fn write_item_collision_uses_fallback_path() {
        let dir = tempdir().unwrap();
        let item = sample_item("bc-col", "Collision Title");
        // Write once to occupy the base slug path.
        let p1 = write_item(dir.path(), &item).unwrap();
        // A second item with the same title gets a different file.
        let item2 = sample_item("bc-c2x", "Collision Title");
        let p2 = write_item(dir.path(), &item2).unwrap();
        assert_ne!(p1, p2);
        assert!(p2.to_string_lossy().contains("c2x"));
    }

    // --- load_all ---

    #[test]
    fn load_all_returns_all_items() {
        let dir = tempdir().unwrap();
        write_item(dir.path(), &sample_item("bc-aa1", "Alpha")).unwrap();
        write_item(dir.path(), &sample_item("bc-bb2", "Beta")).unwrap();
        let items = load_all(dir.path()).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn load_all_skips_non_md_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("notes.txt"), "not an item").unwrap();
        write_item(dir.path(), &sample_item("bc-cc3", "Gamma")).unwrap();
        let items = load_all(dir.path()).unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn load_all_sorted_by_id() {
        let dir = tempdir().unwrap();
        write_item(dir.path(), &sample_item("bc-zzz", "Last")).unwrap();
        write_item(dir.path(), &sample_item("bc-aaa", "First")).unwrap();
        let items = load_all(dir.path()).unwrap();
        assert_eq!(items[0].1.id, "bc-aaa");
        assert_eq!(items[1].1.id, "bc-zzz");
    }

    // --- reindex ---

    #[test]
    fn reindex_creates_csv() {
        let dir = tempdir().unwrap();
        write_item(dir.path(), &sample_item("bc-r01", "Reindex Me")).unwrap();
        reindex(dir.path()).unwrap();
        let csv_path = dir.path().join("index.csv");
        assert!(csv_path.exists());
        let content = std::fs::read_to_string(&csv_path).unwrap();
        assert!(content.contains("bc-r01"));
        assert!(content.contains("Reindex Me"));
    }

    #[test]
    fn reindex_csv_has_header() {
        let dir = tempdir().unwrap();
        reindex(dir.path()).unwrap();
        let content = std::fs::read_to_string(dir.path().join("index.csv")).unwrap();
        assert!(content.starts_with("id,title,status"));
    }

    #[test]
    fn reindex_csv_has_blocks_and_blocked_by_headers() {
        let dir = tempdir().unwrap();
        reindex(dir.path()).unwrap();
        let mut rdr = csv::Reader::from_path(dir.path().join("index.csv")).unwrap();
        let headers = rdr.headers().unwrap().clone();
        let cols: Vec<&str> = headers.iter().collect();
        assert!(
            !cols.contains(&"dependencies"),
            "dependencies column should be removed, got: {cols:?}"
        );
        assert!(
            cols.contains(&"blocks"),
            "expected blocks column, got: {cols:?}"
        );
        assert!(
            cols.contains(&"blocked_by"),
            "expected blocked_by column, got: {cols:?}"
        );
    }

    #[test]
    fn reindex_csv_writes_blocks_and_blocked_by_values() {
        let dir = tempdir().unwrap();
        let item = Item {
            blocks: vec!["cr-aaa".to_string(), "cr-bbb".to_string()],
            blocked_by: vec!["cr-zzz".to_string()],
            ..sample_item("cr-x01", "Blocker Item")
        };
        write_item(dir.path(), &item).unwrap();
        reindex(dir.path()).unwrap();

        let index_path = dir.path().join("index.csv");
        let mut rdr = csv::Reader::from_path(&index_path).unwrap();
        let headers = rdr.headers().unwrap().clone();
        let col = |name: &str| headers.iter().position(|h| h == name).unwrap();
        let blocks_idx = col("blocks");
        let blocked_by_idx = col("blocked_by");

        let row = rdr
            .records()
            .map(|r| r.unwrap())
            .find(|r| r.get(col("id")) == Some("cr-x01"))
            .expect("row for cr-x01 not found");

        assert_eq!(
            row.get(blocks_idx),
            Some("cr-aaa|cr-bbb"),
            "wrong blocks value"
        );
        assert_eq!(
            row.get(blocked_by_idx),
            Some("cr-zzz"),
            "wrong blocked_by value"
        );
    }

    #[test]
    fn reindex_and_export_csv_headers_match() {
        // Ensures the two CSV writers never drift out of sync.
        let dir = tempdir().unwrap();
        reindex(dir.path()).unwrap();
        let mut rdr = csv::Reader::from_path(dir.path().join("index.csv")).unwrap();
        let reindex_headers: Vec<String> =
            rdr.headers().unwrap().iter().map(str::to_owned).collect();

        let export_csv = crate::commands::export::items_to_string(&[], "csv").unwrap();
        let mut export_rdr = csv::Reader::from_reader(export_csv.as_bytes());
        let export_headers: Vec<String> = export_rdr
            .headers()
            .unwrap()
            .iter()
            .map(str::to_owned)
            .collect();

        assert_eq!(
            reindex_headers, export_headers,
            "reindex and export CSV headers diverged"
        );
    }

    // --- find_by_id ---

    #[test]
    fn find_by_id_found() {
        let dir = tempdir().unwrap();
        write_item(dir.path(), &sample_item("bc-f01", "Find Me")).unwrap();
        let result = find_by_id(dir.path(), "bc-f01").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().1.title, "Find Me");
    }

    #[test]
    fn find_by_id_not_found() {
        let dir = tempdir().unwrap();
        let result = find_by_id(dir.path(), "bc-nope").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn find_by_id_case_insensitive() {
        let dir = tempdir().unwrap();
        write_item(dir.path(), &sample_item("bc-f01", "Case Test")).unwrap();
        let result = find_by_id(dir.path(), "BC-F01").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().1.title, "Case Test");
    }

    #[test]
    fn find_by_id_bare_suffix_expands_with_prefix() {
        let dir = tempdir().unwrap();
        // Write a crumbs.toml so the prefix is "bc" (matches item IDs used here).
        crate::store_config::save(
            dir.path(),
            &crate::store_config::StoreConfig {
                prefix: "bc".to_string(),
            },
        )
        .unwrap();
        write_item(dir.path(), &sample_item("bc-s01", "Bare Suffix")).unwrap();
        // "s01" (no dash) should expand to "bc-s01".
        let result = find_by_id(dir.path(), "s01").unwrap();
        assert!(
            result.is_some(),
            "bare suffix lookup should find bc-s01 when prefix is 'bc'"
        );
        assert_eq!(result.unwrap().1.title, "Bare Suffix");
    }

    #[test]
    fn find_by_id_bare_suffix_case_insensitive() {
        let dir = tempdir().unwrap();
        crate::store_config::save(
            dir.path(),
            &crate::store_config::StoreConfig {
                prefix: "bc".to_string(),
            },
        )
        .unwrap();
        write_item(dir.path(), &sample_item("bc-s02", "Case Bare")).unwrap();
        // Upper-cased bare suffix should still match.
        let result = find_by_id(dir.path(), "S02").unwrap();
        assert!(
            result.is_some(),
            "bare suffix lookup should be case-insensitive"
        );
        assert_eq!(result.unwrap().1.title, "Case Bare");
    }

    #[test]
    fn find_by_id_bare_suffix_wrong_prefix_returns_none() {
        let dir = tempdir().unwrap();
        // Store has prefix "bc" but item id uses "xx".
        crate::store_config::save(
            dir.path(),
            &crate::store_config::StoreConfig {
                prefix: "bc".to_string(),
            },
        )
        .unwrap();
        write_item(dir.path(), &sample_item("xx-s03", "Wrong Prefix")).unwrap();
        // "s03" with prefix "bc" gives "bc-s03" which doesn't match "xx-s03".
        let result = find_by_id(dir.path(), "s03").unwrap();
        assert!(
            result.is_none(),
            "bare suffix with mismatched prefix should return None"
        );
    }
}
