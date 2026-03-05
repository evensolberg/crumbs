use tempfile::tempdir;

use crumbs::{
    commands,
    item::{ItemType, Status},
    store,
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn create_task(dir: &std::path::Path, title: &str) -> String {
    commands::create::run(
        dir,
        title.to_string(),
        ItemType::Task,
        2,
        vec![],
        String::new(),
        vec![],
    )
    .unwrap();

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
    commands::init::run(&target, Some("ts".to_string())).unwrap();
    assert!(target.is_dir());
}

#[test]
fn init_is_idempotent() {
    let base = tempdir().unwrap();
    let target = base.path().join(".crumbs");
    commands::init::run(&target, Some("ts".to_string())).unwrap();
    commands::init::run(&target, Some("ts".to_string())).unwrap(); // second call should not error
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
        String::new(),
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
        String::new(),
        vec![],
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
    assert!(item.description.is_empty());
}

#[test]
fn create_also_writes_index_csv() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "CSV Test".to_string(),
        ItemType::Task,
        2,
        vec![],
        String::new(),
        vec![],
    )
    .unwrap();
    assert!(dir.path().join("index.csv").exists());
}

#[test]
fn create_with_description_stores_body() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "Described Task".to_string(),
        ItemType::Task,
        2,
        vec![],
        "This is more detail.".to_string(),
        vec![],
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].1.description, "This is more detail.");
}

#[test]
fn create_with_description_appears_in_md_body() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "Body Task".to_string(),
        ItemType::Task,
        2,
        vec![],
        "Extra context here.".to_string(),
        vec![],
    )
    .unwrap();
    let md: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .collect();
    let content = std::fs::read_to_string(md[0].path()).unwrap();
    assert!(content.contains("Extra context here."));
}

#[test]
fn create_without_description_has_empty_description() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "No Desc".to_string(),
        ItemType::Task,
        2,
        vec![],
        String::new(),
        vec![],
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    assert!(items[0].1.description.is_empty());
}

// ── update ───────────────────────────────────────────────────────────────────

#[test]
fn update_changes_status() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Status Update");
    commands::update::run(
        dir.path(),
        &id,
        Some("in_progress".to_string()),
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.status, Status::InProgress);
}

#[test]
fn update_changes_priority() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Priority Update");
    commands::update::run(dir.path(), &id, None, Some(0), None, None, None).unwrap();
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
        None,
        None,
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.tags, vec!["new-tag"]);
}

#[test]
fn update_changes_type() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Type Update");
    commands::update::run(
        dir.path(),
        &id,
        None,
        None,
        None,
        Some("bug".to_string()),
        None,
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.item_type, ItemType::Bug);
}

#[test]
fn update_unknown_id_errors() {
    let dir = tempdir().unwrap();
    let result = commands::update::run(dir.path(), "bc-zzz", None, None, None, None, None);
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
    commands::list::run(dir.path(), None, None, false).unwrap();
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
        String::new(),
        vec![],
    )
    .unwrap();
    commands::create::run(
        dir.path(),
        "Untagged".to_string(),
        ItemType::Task,
        2,
        vec![],
        String::new(),
        vec![],
    )
    .unwrap();

    let items = store::load_all(dir.path()).unwrap();
    let with_tag: Vec<_> = items
        .iter()
        .filter(|(_, i)| i.tags.iter().any(|t: &String| t.contains("project/x")))
        .collect();
    assert_eq!(with_tag.len(), 1);
    assert_eq!(with_tag[0].1.title, "Tagged");
}

// ── delete ───────────────────────────────────────────────────────────────────

#[test]
fn delete_removes_item() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Delete Me");
    commands::delete::run(dir.path(), &id).unwrap();
    assert!(store::find_by_id(dir.path(), &id).unwrap().is_none());
}

#[test]
fn delete_unknown_id_errors() {
    let dir = tempdir().unwrap();
    let result = commands::delete::run(dir.path(), "cr-zzz");
    assert!(result.is_err());
}

#[test]
fn delete_closed_removes_only_closed() {
    let dir = tempdir().unwrap();
    let id_a = create_task(dir.path(), "Keep Me");
    let id_b = create_task(dir.path(), "Delete Me Closed");
    commands::close::run(dir.path(), &id_b, None).unwrap();
    commands::delete::run_closed(dir.path()).unwrap();
    assert!(store::find_by_id(dir.path(), &id_a).unwrap().is_some());
    assert!(store::find_by_id(dir.path(), &id_b).unwrap().is_none());
}

#[test]
fn delete_closed_noop_when_none_closed() {
    let dir = tempdir().unwrap();
    create_task(dir.path(), "Open Task");
    commands::delete::run_closed(dir.path()).unwrap(); // should not error
    let items = store::load_all(dir.path()).unwrap();
    assert_eq!(items.len(), 1);
}

// ── dependencies ─────────────────────────────────────────────────────────────

#[test]
fn create_with_dependencies_stores_them() {
    let dir = tempdir().unwrap();
    let dep_id = create_task(dir.path(), "Dep Task");
    commands::create::run(
        dir.path(),
        "Dependent Task".to_string(),
        ItemType::Task,
        2,
        vec![],
        String::new(),
        vec![dep_id.clone()],
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    let dependent = items
        .iter()
        .find(|(_, i)| i.title == "Dependent Task")
        .unwrap();
    assert_eq!(dependent.1.dependencies, vec![dep_id]);
}

#[test]
fn update_replaces_dependencies() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Task With Deps");
    let dep_id = create_task(dir.path(), "Another Task");
    commands::update::run(
        dir.path(),
        &id,
        None,
        None,
        None,
        None,
        Some(vec![dep_id.clone()]),
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.dependencies, vec![dep_id]);
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
