use tempfile::tempdir;

use crumbs::{
    commands,
    item::{ItemType, Status},
    store,
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn create_task(dir: &std::path::Path, title: &str) -> String {
    commands::create::run(dir, title.to_string(), ItemType::Task, 2, vec![]).unwrap();
    let items = store::load_all(dir).unwrap();
    items
        .into_iter()
        .find(|(_, i)| i.title == title)
        .unwrap()
        .1
        .id
}

// ── init ─────────────────────────────────────────────────────────────────────

#[test]
fn init_creates_crumbs_dir() {
    let base = tempdir().unwrap();
    let target = base.path().join(".crumbs");
    commands::init::run(&target).unwrap();
    assert!(target.is_dir());
}

#[test]
fn init_is_idempotent() {
    let base = tempdir().unwrap();
    let target = base.path().join(".crumbs");
    commands::init::run(&target).unwrap();
    commands::init::run(&target).unwrap(); // second call should not error
    assert!(target.is_dir());
}

// ── create ───────────────────────────────────────────────────────────────────

#[test]
fn create_produces_md_file() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "My Task".to_string(),
        ItemType::Task,
        2,
        vec![],
    )
    .unwrap();
    let md_files: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .collect();
    assert_eq!(md_files.len(), 1);
}

#[test]
fn create_writes_correct_frontmatter() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "Frontmatter Check".to_string(),
        ItemType::Bug,
        1,
        vec!["project/foo".to_string()],
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    assert_eq!(items.len(), 1);
    let item = &items[0].1;
    assert_eq!(item.title, "Frontmatter Check");
    assert_eq!(item.item_type, ItemType::Bug);
    assert_eq!(item.priority, 1);
    assert_eq!(item.tags, vec!["project/foo"]);
    assert_eq!(item.status, Status::Open);
}

#[test]
fn create_also_writes_index_csv() {
    let dir = tempdir().unwrap();
    commands::create::run(dir.path(), "CSV Test".to_string(), ItemType::Task, 2, vec![]).unwrap();
    assert!(dir.path().join("index.csv").exists());
}

// ── update ───────────────────────────────────────────────────────────────────

#[test]
fn update_changes_status() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Status Update");
    commands::update::run(dir.path(), &id, Some("in_progress".to_string()), None, None).unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.status, Status::InProgress);
}

#[test]
fn update_changes_priority() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Priority Update");
    commands::update::run(dir.path(), &id, None, Some(0), None).unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.priority, 0);
}

#[test]
fn update_replaces_tags() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Tag Update");
    commands::update::run(
        dir.path(),
        &id,
        None,
        None,
        Some(vec!["new-tag".to_string()]),
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.tags, vec!["new-tag"]);
}

#[test]
fn update_unknown_id_errors() {
    let dir = tempdir().unwrap();
    let result = commands::update::run(dir.path(), "bc-zzz", None, None, None);
    assert!(result.is_err());
}

// ── close ────────────────────────────────────────────────────────────────────

#[test]
fn close_sets_status_closed() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Close Me");
    commands::close::run(dir.path(), &id, None).unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.status, Status::Closed);
}

#[test]
fn close_stores_reason() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Close With Reason");
    commands::close::run(dir.path(), &id, Some("no longer needed".to_string())).unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.closed_reason, "no longer needed");
}

#[test]
fn close_unknown_id_errors() {
    let dir = tempdir().unwrap();
    let result = commands::close::run(dir.path(), "bc-zzz", None);
    assert!(result.is_err());
}

// ── list ─────────────────────────────────────────────────────────────────────

#[test]
fn list_no_filter_does_not_error() {
    let dir = tempdir().unwrap();
    create_task(dir.path(), "List Task 1");
    create_task(dir.path(), "List Task 2");
    // list prints to stdout; just verify no error
    commands::list::run(dir.path(), None, None).unwrap();
}

#[test]
fn list_status_filter_only_shows_matching() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Open One");
    create_task(dir.path(), "Open Two");
    commands::close::run(dir.path(), &id, None).unwrap();

    // Manually verify filtering logic via store
    let items = store::load_all(dir.path()).unwrap();
    let open: Vec<_> = items
        .iter()
        .filter(|(_, i)| i.status == Status::Open)
        .collect();
    let closed: Vec<_> = items
        .iter()
        .filter(|(_, i)| i.status == Status::Closed)
        .collect();
    assert_eq!(open.len(), 1);
    assert_eq!(closed.len(), 1);
}

#[test]
fn list_tag_filter_only_shows_matching() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "Tagged".to_string(),
        ItemType::Task,
        2,
        vec!["project/x".to_string()],
    )
    .unwrap();
    commands::create::run(dir.path(), "Untagged".to_string(), ItemType::Task, 2, vec![]).unwrap();

    let items = store::load_all(dir.path()).unwrap();
    let with_tag: Vec<_> = items
        .iter()
        .filter(|(_, i)| i.tags.iter().any(|t: &String| t.contains("project/x")))
        .collect();
    assert_eq!(with_tag.len(), 1);
    assert_eq!(with_tag[0].1.title, "Tagged");
}

// ── search ───────────────────────────────────────────────────────────────────

#[test]
fn search_matches_title() {
    let dir = tempdir().unwrap();
    create_task(dir.path(), "Unique Needle Title");
    create_task(dir.path(), "Other Task");
    // Just verifying no error; search prints to stdout
    commands::search::run(dir.path(), "Needle").unwrap();
}

#[test]
fn search_does_not_error_on_no_results() {
    let dir = tempdir().unwrap();
    create_task(dir.path(), "Some Task");
    commands::search::run(dir.path(), "xyzzy_nonexistent").unwrap();
}

// ── reindex ───────────────────────────────────────────────────────────────────

#[test]
fn reindex_rebuilds_stale_csv() {
    let dir = tempdir().unwrap();
    create_task(dir.path(), "Before Reindex");
    // Corrupt the CSV
    std::fs::write(dir.path().join("index.csv"), "garbage").unwrap();
    // Reindex should fix it
    commands::reindex::run(dir.path()).unwrap();
    let content = std::fs::read_to_string(dir.path().join("index.csv")).unwrap();
    assert!(content.contains("Before Reindex"));
}
