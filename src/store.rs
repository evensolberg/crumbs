use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use slugify::slugify;

use crate::item::Item;

pub fn item_path(dir: &Path, item: &Item) -> PathBuf {
    let slug = slugify!(&item.title, max_length = 60);
    dir.join(format!("{slug}.md"))
}

/// Resolve a unique file path, appending the ID suffix on collision.
pub fn unique_path(dir: &Path, item: &Item) -> PathBuf {
    let base = item_path(dir, item);
    if !base.exists() {
        return base;
    }
    let slug = slugify!(&item.title, max_length = 50);
    let id_suffix = item.id.trim_start_matches("bc-");
    dir.join(format!("{slug}-{id_suffix}.md"))
}

pub fn write_item(dir: &Path, item: &Item) -> Result<PathBuf> {
    let path = unique_path(dir, item);
    let frontmatter = serde_yaml::to_string(item).context("serialize frontmatter")?;
    let content = format!("---\n{frontmatter}---\n\n# {}\n", item.title);
    std::fs::write(&path, content).context("write item file")?;
    Ok(path)
}

pub fn read_item(path: &Path) -> Result<Item> {
    let raw = std::fs::read_to_string(path).context("read item file")?;
    parse_item(&raw).context("parse item")
}

pub fn parse_item(raw: &str) -> Result<Item> {
    let inner = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(fm, _)| fm))
        .ok_or_else(|| anyhow!("missing frontmatter delimiters"))?;
    let item: Item = serde_yaml::from_str(inner).context("deserialize frontmatter")?;
    Ok(item)
}

pub fn load_all(dir: &Path) -> Result<Vec<(PathBuf, Item)>> {
    let mut items = Vec::new();
    for entry in std::fs::read_dir(dir).context("read dir")? {
        let entry: std::fs::DirEntry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e: &std::ffi::OsStr| e.to_str()) == Some("md") {
            if path.file_name().and_then(|n: &std::ffi::OsStr| n.to_str()) == Some("index.csv") {
                continue;
            }
            match read_item(&path) {
                Ok(item) => items.push((path, item)),
                Err(e) => eprintln!("warning: skipping {}: {e}", path.display()),
            }
        }
    }
    items.sort_by(|a, b| a.1.id.cmp(&b.1.id));
    Ok(items)
}

pub fn reindex(dir: &Path) -> Result<()> {
    let items = load_all(dir)?;
    let index_path = dir.join("index.csv");
    let mut wtr = csv::Writer::from_path(&index_path).context("open index.csv")?;
    wtr.write_record([
        "id", "title", "status", "type", "priority", "tags", "created", "updated",
        "closed_reason",
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
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn find_by_id(dir: &Path, id: &str) -> Result<Option<(PathBuf, Item)>> {
    let items = load_all(dir)?;
    Ok(items.into_iter().find(|(_, item)| item.id == id))
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
    fn unique_path_collision_appends_id() {
        let dir = tempdir().unwrap();
        let item = sample_item("bc-col", "Collision Title");
        // Write once to create the base slug file
        write_item(dir.path(), &item).unwrap();
        // Second call should produce a different path
        let p1 = item_path(dir.path(), &item);
        let p2 = unique_path(dir.path(), &item);
        assert_ne!(p1, p2);
        assert!(p2.to_string_lossy().contains("col"));
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
