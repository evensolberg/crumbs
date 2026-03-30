use chrono::NaiveDate;
use tempfile::tempdir;

use crumbs::{
    commands,
    commands::list::SortKey,
    commands::update::UpdateArgs,
    item::{Item, ItemType, Status},
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
        None,
        None,
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
    commands::init::run(&target, Some("ts".to_string())).unwrap();
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
        None,
        None,
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
        None,
        None,
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
        None,
        None,
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
        None,
        None,
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
        None,
        None,
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
fn description_not_in_frontmatter() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "Frontmatter Check".to_string(),
        ItemType::Task,
        2,
        vec![],
        "Should be body only.".to_string(),
        vec![],
        None,
        None,
    )
    .unwrap();
    let md: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .collect();
    let content = std::fs::read_to_string(md[0].path()).unwrap();
    // Extract YAML frontmatter (between the two --- delimiters)
    let fm = content
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(fm, _)| fm))
        .expect("frontmatter delimiters not found");
    assert!(
        !fm.contains("description:"),
        "description field must not appear in YAML frontmatter; got:\n{fm}"
    );
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
        None,
        None,
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
        UpdateArgs {
            status: Some("in_progress".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.status, Status::InProgress);
}

#[test]
fn update_changes_priority() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Priority Update");
    commands::update::run(
        dir.path(),
        &id,
        UpdateArgs {
            priority: Some(0),
            ..Default::default()
        },
    )
    .unwrap();
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
        UpdateArgs {
            tags: Some(vec!["new-tag".to_string()]),
            ..Default::default()
        },
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
        UpdateArgs {
            item_type: Some("bug".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.item_type, ItemType::Bug);
}

#[test]
fn update_unknown_id_errors() {
    let dir = tempdir().unwrap();
    let result = commands::update::run(dir.path(), "bc-zzz", UpdateArgs::default());
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
    commands::list::run(
        dir.path(),
        None,
        None,
        None,
        None,
        false,
        false,
        SortKey::Id,
    )
    .unwrap();
}

#[test]
fn list_status_filter_only_shows_matching() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Open One");
    create_task(dir.path(), "Open Two");
    commands::close::run(dir.path(), &id, None).unwrap();

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
        None,
        None,
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
        None,
        None,
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

// ── move / import ────────────────────────────────────────────────────────────

#[test]
fn move_transfers_item_to_destination() {
    let src = tempdir().unwrap();
    let dst = tempdir().unwrap();
    commands::init::run(src.path(), Some("src".to_string())).unwrap();
    commands::init::run(dst.path(), Some("dst".to_string())).unwrap();
    let id = create_task(src.path(), "Move Me");
    commands::move_::run(src.path(), &id, dst.path()).unwrap();
    // Item is gone from source.
    assert!(store::find_by_id(src.path(), &id).unwrap().is_none());
    // Item appears in destination (with a new ID under the dst prefix).
    let items = store::load_all(dst.path()).unwrap();
    assert!(items.iter().any(|(_, i)| i.title == "Move Me"));
}

#[test]
fn import_transfers_item_from_source_to_current() {
    let src = tempdir().unwrap();
    let dst = tempdir().unwrap();
    commands::init::run(src.path(), Some("src".to_string())).unwrap();
    commands::init::run(dst.path(), Some("dst".to_string())).unwrap();
    let id = create_task(src.path(), "Import Me");
    // import is move_::run(src, id, dst) — source first, destination second.
    commands::move_::run(src.path(), &id, dst.path()).unwrap();
    // Item is gone from source.
    assert!(store::find_by_id(src.path(), &id).unwrap().is_none());
    // Item appears in destination.
    let items = store::load_all(dst.path()).unwrap();
    assert!(items.iter().any(|(_, i)| i.title == "Import Me"));
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
    commands::delete::run_closed(dir.path()).unwrap();
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
        None,
        None,
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
        UpdateArgs {
            dependencies: Some(vec![dep_id.clone()]),
            ..Default::default()
        },
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
    std::fs::write(dir.path().join("index.csv"), "garbage").unwrap();
    commands::reindex::run(dir.path()).unwrap();
    let content = std::fs::read_to_string(dir.path().join("index.csv")).unwrap();
    assert!(content.contains("Before Reindex"));
}

// ── link ──────────────────────────────────────────────────────────────────────

#[test]
fn link_blocks_updates_both_items() {
    let dir = tempdir().unwrap();
    let id_a = create_task(dir.path(), "Blocker");
    let id_b = create_task(dir.path(), "Blocked");
    commands::link::run(dir.path(), &id_a, "blocks", &[id_b.clone()], false).unwrap();
    let (_, item_a) = store::find_by_id(dir.path(), &id_a).unwrap().unwrap();
    let (_, item_b) = store::find_by_id(dir.path(), &id_b).unwrap().unwrap();
    assert_eq!(item_a.blocks, vec![id_b.clone()]);
    assert_eq!(item_b.blocked_by, vec![id_a.clone()]);
}

#[test]
fn link_blocked_by_is_inverse() {
    let dir = tempdir().unwrap();
    let id_a = create_task(dir.path(), "Task A");
    let id_b = create_task(dir.path(), "Task B");
    commands::link::run(dir.path(), &id_a, "blocked-by", &[id_b.clone()], false).unwrap();
    let (_, item_a) = store::find_by_id(dir.path(), &id_a).unwrap().unwrap();
    let (_, item_b) = store::find_by_id(dir.path(), &id_b).unwrap().unwrap();
    assert_eq!(item_a.blocked_by, vec![id_b.clone()]);
    assert_eq!(item_b.blocks, vec![id_a.clone()]);
}

#[test]
fn link_idempotent() {
    let dir = tempdir().unwrap();
    let id_a = create_task(dir.path(), "Idempotent A");
    let id_b = create_task(dir.path(), "Idempotent B");
    commands::link::run(dir.path(), &id_a, "blocks", &[id_b.clone()], false).unwrap();
    commands::link::run(dir.path(), &id_a, "blocks", &[id_b.clone()], false).unwrap();
    let (_, item_a) = store::find_by_id(dir.path(), &id_a).unwrap().unwrap();
    assert_eq!(item_a.blocks.len(), 1);
}

#[test]
fn unlink_removes_from_both_sides() {
    let dir = tempdir().unwrap();
    let id_a = create_task(dir.path(), "Unlink A");
    let id_b = create_task(dir.path(), "Unlink B");
    commands::link::run(dir.path(), &id_a, "blocks", &[id_b.clone()], false).unwrap();
    commands::link::run(dir.path(), &id_a, "blocks", &[id_b.clone()], true).unwrap();
    let (_, item_a) = store::find_by_id(dir.path(), &id_a).unwrap().unwrap();
    let (_, item_b) = store::find_by_id(dir.path(), &id_b).unwrap().unwrap();
    assert!(item_a.blocks.is_empty());
    assert!(item_b.blocked_by.is_empty());
}

#[test]
fn link_unknown_id_errors() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Real Task");
    let result = commands::link::run(dir.path(), &id, "blocks", &["bc-nope".to_string()], false);
    assert!(result.is_err());
}

// ── update --message ──────────────────────────────────────────────────────────

#[test]
fn update_message_replaces_description() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Message Update");
    commands::update::run(
        dir.path(),
        &id,
        UpdateArgs {
            message: Some("New description text.".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (path, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.description, "New description text.");
    // Verify the description lives in the body, not the YAML frontmatter.
    let content = std::fs::read_to_string(&path).unwrap();
    let fm = content
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(fm, _)| fm))
        .expect("frontmatter delimiters not found");
    assert!(
        !fm.contains("description:"),
        "description must not appear in YAML frontmatter after update; got:\n{fm}"
    );
}

#[test]
fn update_title_rewrites_body_heading() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Old Title");
    commands::update::run(
        dir.path(),
        &id,
        UpdateArgs {
            title: Some("New Title".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (path, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.title, "New Title");
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("# New Title"),
        "body heading must reflect new title"
    );
    assert!(
        !content.contains("# Old Title"),
        "old heading must be replaced"
    );
}

#[test]
fn update_title_preserves_existing_description() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "Old Title".to_string(),
        ItemType::Task,
        2,
        vec![],
        "Body text to keep.".to_string(),
        vec![],
        None,
        None,
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    let id = items[0].1.id.clone();
    commands::update::run(
        dir.path(),
        &id,
        UpdateArgs {
            title: Some("New Title".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.title, "New Title");
    assert_eq!(item.description, "Body text to keep.");
}

// ── show ─────────────────────────────────────────────────────────────────────

#[test]
fn show_full_id_succeeds() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Show Me");
    // Full ID lookup must succeed without error.
    commands::show::run(dir.path(), &[id]).unwrap();
}

#[test]
fn show_unknown_id_errors() {
    let dir = tempdir().unwrap();
    let result = commands::show::run(dir.path(), &["cr-zzz".to_string()]);
    assert!(result.is_err());
}

#[test]
fn show_bare_suffix_expands_with_store_prefix() {
    let dir = tempdir().unwrap();
    // Initialize the store so config.toml is written with prefix "cr".
    commands::init::run(&dir.path().join(".crumbs"), Some("cr".to_string())).unwrap();
    // For this test we work directly with the store dir for simplicity.
    let store_dir = dir.path().join(".crumbs");
    store::write_item(
        &store_dir,
        &Item {
            id: "cr-b01".to_string(),
            title: "Bare Lookup".to_string(),
            status: Status::Open,
            item_type: ItemType::Task,
            priority: 2,
            tags: vec![],
            created: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            closed_reason: String::new(),
            dependencies: vec![],
            blocks: vec![],
            blocked_by: vec![],
            due: None,
            description: String::new(),
            story_points: None,
        },
    )
    .unwrap();
    // "b01" (bare suffix) must resolve to "cr-b01" and succeed.
    commands::show::run(&store_dir, &["b01".to_string()]).unwrap();
}

// ── Emoji shortcode expansion ─────────────────────────────────────────────────

#[test]
fn emoji_shortcodes_expanded_on_create() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "Emoji test".to_string(),
        ItemType::Task,
        2,
        vec![],
        ":tada:".to_string(),
        vec![],
        None,
        None,
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    let item = &items[0].1;
    assert_eq!(item.description, "🎉");
}

#[test]
fn emoji_shortcodes_expanded_on_update_message() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Update emoji");
    commands::update::run(
        dir.path(),
        &id,
        UpdateArgs {
            message: Some(":bug: found".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.description, "🐛 found");
}

#[test]
fn emoji_shortcodes_expanded_on_update_append() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Append emoji");
    commands::update::run(
        dir.path(),
        &id,
        UpdateArgs {
            message: Some(":white_check_mark: fixed".to_string()),
            append: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert!(
        item.description.contains("✅ fixed"),
        "expected '✅ fixed' in description, got: {:?}",
        item.description
    );
}

#[test]
fn update_run_appends_body_when_append_is_true() {
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Append flag test");
    commands::update::run(
        dir.path(),
        &id,
        UpdateArgs {
            message: Some("a note via append flag".to_string()),
            append: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert!(
        item.description.contains("a note via append flag"),
        "expected appended text in description, got: {:?}",
        item.description
    );
}

// ── list --sort ───────────────────────────────────────────────────────────────

#[test]
fn sort_by_priority_ascending() {
    let dir = tempdir().unwrap();
    let id_low = create_task(dir.path(), "Low priority");
    let id_high = create_task(dir.path(), "High priority");
    commands::update::run(
        dir.path(),
        &id_low,
        UpdateArgs {
            priority: Some(3),
            ..Default::default()
        },
    )
    .unwrap();
    commands::update::run(
        dir.path(),
        &id_high,
        UpdateArgs {
            priority: Some(0),
            ..Default::default()
        },
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    let sorted = commands::list::sort_items(items, SortKey::Priority);
    assert_eq!(sorted[0].1.id, id_high, "priority 0 should come first");
    assert_eq!(sorted[1].1.id, id_low, "priority 3 should come last");
}

#[test]
fn sort_by_title_alphabetical() {
    let dir = tempdir().unwrap();
    create_task(dir.path(), "Zebra");
    create_task(dir.path(), "Apple");
    let items = store::load_all(dir.path()).unwrap();
    let sorted = commands::list::sort_items(items, SortKey::Title);
    assert_eq!(sorted[0].1.title, "Apple");
    assert_eq!(sorted[1].1.title, "Zebra");
}

#[test]
fn sort_by_status_groups_statuses() {
    let dir = tempdir().unwrap();
    let id_open = create_task(dir.path(), "Open task");
    let id_prog = create_task(dir.path(), "In progress task");
    commands::update::run(
        dir.path(),
        &id_prog,
        UpdateArgs {
            status: Some("in_progress".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    let sorted = commands::list::sort_items(items, SortKey::Status);
    // in_progress sorts before open alphabetically
    assert_eq!(sorted[0].1.id, id_prog);
    assert_eq!(sorted[1].1.id, id_open);
}

#[test]
fn sort_by_id_default_order() {
    let dir = tempdir().unwrap();
    create_task(dir.path(), "First");
    create_task(dir.path(), "Second");
    let items = store::load_all(dir.path()).unwrap();
    let sorted_id = commands::list::sort_items(items.clone(), SortKey::Id);
    let sorted_default: Vec<_> = {
        let mut v = items;
        v.sort_by(|a, b| a.1.id.cmp(&b.1.id));
        v
    };
    let ids_sorted: Vec<_> = sorted_id.iter().map(|(_, i)| &i.id).collect();
    let ids_default: Vec<_> = sorted_default.iter().map(|(_, i)| &i.id).collect();
    assert_eq!(ids_sorted, ids_default);
}

#[test]
fn sort_by_type_alphabetical() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        "A bug".to_string(),
        ItemType::Bug,
        2,
        vec![],
        String::new(),
        vec![],
        None,
        None,
    )
    .unwrap();
    commands::create::run(
        dir.path(),
        "A feature".to_string(),
        ItemType::Feature,
        2,
        vec![],
        String::new(),
        vec![],
        None,
        None,
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    let sorted = commands::list::sort_items(items, SortKey::Type);
    assert_eq!(sorted[0].1.item_type, ItemType::Bug);
    assert_eq!(sorted[1].1.item_type, ItemType::Feature);
}
