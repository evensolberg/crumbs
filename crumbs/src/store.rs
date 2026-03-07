use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use slugify::slugify;

use crate::item::Item;

/// Atomically overwrite `path` with `content`.
///
/// Writes to a sibling temp file then renames, so a mid-write crash or kill
/// leaves the original file intact.
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

pub fn item_path(dir: &Path, item: &Item) -> PathBuf {
    let slug = slugify!(&item.title, max_length = 60);
    dir.join(format!("{slug}.md"))
}

/// Build the fallback path (slug + ID suffix) used when the base slug collides.
fn fallback_path(dir: &Path, item: &Item) -> PathBuf {
    let slug = slugify!(&item.title, max_length = 50);
    let id_suffix = item.id.split_once('-').map(|x| x.1).unwrap_or(&item.id);
    dir.join(format!("{slug}-{id_suffix}.md"))
}

pub fn write_item(dir: &Path, item: &Item) -> Result<PathBuf> {
    let frontmatter = serde_yaml_ng::to_string(item).context("serialize frontmatter")?;
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

pub fn read_item(path: &Path) -> Result<Item> {
    let raw = std::fs::read_to_string(path).context("read item file")?;
    parse_item(&raw).context("parse item")
}

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
    items.sort_by(|a, b| a.1.id.cmp(&b.1.id));
    Ok(items)
}

pub fn reindex(dir: &Path) -> Result<()> {
    let items = load_all(dir)?;
    let index_path = dir.join("index.csv");
    let mut wtr = csv::Writer::from_path(&index_path).context("open index.csv")?;
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
        "story_points",
    ])?;
    for (_, item) in &items {
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
            &item
                .story_points
                .map(|sp| sp.to_string())
                .unwrap_or_default(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

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
}
