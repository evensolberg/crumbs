use assert_cmd::Command;
use chrono::NaiveDate;
use tempfile::tempdir;

use crumbs::{
    commands,
    commands::create::CreateArgs,
    commands::list::{ListArgs, SortKey},
    commands::update::UpdateArgs,
    item::{Item, ItemType, Status},
    store,
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn create_task(dir: &std::path::Path, title: &str) -> String {
    commands::create::run(
        dir,
        CreateArgs {
            title: title.to_string(),
            ..Default::default()
        },
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

// ── create (CreateArgs) ───────────────────────────────────────────────────────

#[test]
fn create_run_accepts_create_args_struct() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Test via CreateArgs".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    let items = store::load_all(&d).unwrap();
    assert!(
        items
            .into_iter()
            .any(|(_, i)| i.title == "Test via CreateArgs")
    );
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
        CreateArgs {
            title: "My Task".to_string(),
            ..Default::default()
        },
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
        CreateArgs {
            title: "Frontmatter Check".to_string(),
            item_type: ItemType::Bug,
            priority: 1,
            tags: vec!["project/foo".to_string()],
            ..Default::default()
        },
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
        CreateArgs {
            title: "CSV Test".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(dir.path().join("index.csv").exists());
}

#[test]
fn create_with_description_stores_body() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        CreateArgs {
            title: "Described Task".to_string(),
            description: "This is more detail.".to_string(),
            ..Default::default()
        },
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
        CreateArgs {
            title: "Body Task".to_string(),
            description: "Extra context here.".to_string(),
            ..Default::default()
        },
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
        CreateArgs {
            title: "Frontmatter Check".to_string(),
            description: "Should be body only.".to_string(),
            ..Default::default()
        },
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
        CreateArgs {
            title: "No Desc".to_string(),
            ..Default::default()
        },
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
    commands::list::run(dir.path(), ListArgs::default()).unwrap();
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
        CreateArgs {
            title: "Tagged".to_string(),
            tags: vec!["project/x".to_string()],
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        dir.path(),
        CreateArgs {
            title: "Untagged".to_string(),
            ..Default::default()
        },
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
    // init::run returns early if the dir already exists, so point it at the
    // .crumbs subdirectory (which doesn't exist yet) so config.toml is written.
    let src_store = src.path().join(".crumbs");
    let dst_store = dst.path().join(".crumbs");
    commands::init::run(&src_store, Some("src".to_string())).unwrap();
    commands::init::run(&dst_store, Some("dst".to_string())).unwrap();
    let id = create_task(&src_store, "Move Me");
    commands::move_::run(&src_store, &id, &dst_store).unwrap();
    // Item is gone from source.
    assert!(store::find_by_id(&src_store, &id).unwrap().is_none());
    // Item appears in destination (with a new ID under the dst prefix).
    let items = store::load_all(&dst_store).unwrap();
    assert!(items.iter().any(|(_, i)| i.title == "Move Me"));
}

#[test]
fn import_direction_is_src_to_dst_not_dst_to_src() {
    // Regression test for the import CLI dispatch bug: move_::run(&dir, &id, &src)
    // was called with src and dst swapped, moving the item the wrong way.
    // This test verifies that run(src, id, dst) moves the item FROM src TO dst,
    // not from dst to src.
    //
    // init::run returns early if the dir already exists, so point it at the
    // .crumbs subdirectory (which doesn't exist yet) so config.toml is written.
    let src = tempdir().unwrap();
    let dst = tempdir().unwrap();
    let src_store = src.path().join(".crumbs");
    let dst_store = dst.path().join(".crumbs");
    commands::init::run(&src_store, Some("src".to_string())).unwrap();
    commands::init::run(&dst_store, Some("dst".to_string())).unwrap();
    let id = create_task(&src_store, "Import Me");
    // If the args were swapped (bug), run(dst, id, src) would look for "id" in dst
    // (where it doesn't exist) and return an error. Running correctly should succeed.
    commands::move_::run(&src_store, &id, &dst_store).unwrap();
    // Item is gone from source — confirms directionality.
    assert!(
        store::find_by_id(&src_store, &id).unwrap().is_none(),
        "item must leave src"
    );
    // Item appears in destination — confirms it arrived.
    let dst_items = store::load_all(&dst_store).unwrap();
    assert!(
        dst_items.iter().any(|(_, i)| i.title == "Import Me"),
        "item must arrive in dst"
    );
    // Source is empty — nothing was moved into it.
    let src_items = store::load_all(&src_store).unwrap();
    assert!(
        src_items.is_empty(),
        "src must be empty after move (nothing moved into it)"
    );
}

#[test]
fn import_cli_dispatch_moves_item_from_src_to_dst() {
    // Binary-level regression test for the CLI dispatch bug: `Command::Import`
    // in main.rs previously called move_::run(&dir, &id, &src) with src/dst
    // swapped. This test drives the real binary to catch any future regression
    // at the dispatch layer, which the library-level test above cannot reach.
    //
    // `crumbs init` ignores --dir and uses current_dir()/.crumbs, so we set
    // current_dir on those invocations. Subsequent commands use --dir with the
    // .crumbs subdirectory path directly (resolve_dir keeps paths that already
    // end in ".crumbs" unchanged).
    let src = tempdir().unwrap();
    let dst = tempdir().unwrap();
    let src_store = src.path().join(".crumbs");
    let dst_store = dst.path().join(".crumbs");
    // Initialise both stores via the binary (current_dir sets the store root).
    Command::cargo_bin("crumbs")
        .unwrap()
        .current_dir(src.path())
        .args(["init", "--prefix", "src"])
        .assert()
        .success();
    Command::cargo_bin("crumbs")
        .unwrap()
        .current_dir(dst.path())
        .args(["init", "--prefix", "dst"])
        .assert()
        .success();
    // Create an item in src via the binary.
    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            src_store.to_str().unwrap(),
            "create",
            "CLI Import Me",
        ])
        .assert()
        .success();
    // Retrieve the generated ID.
    let src_items = store::load_all(&src_store).unwrap();
    assert_eq!(src_items.len(), 1);
    let id = src_items[0].1.id.clone();
    // Run `crumbs import <id> --from <src_store>` targeting the dst store.
    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            dst_store.to_str().unwrap(),
            "import",
            &id,
            "--from",
            src_store.to_str().unwrap(),
        ])
        .assert()
        .success();
    // Item must be gone from source.
    assert!(
        store::find_by_id(&src_store, &id).unwrap().is_none(),
        "item must leave src after import"
    );
    // Item must appear in destination.
    let dst_items = store::load_all(&dst_store).unwrap();
    assert!(
        dst_items.iter().any(|(_, i)| i.title == "CLI Import Me"),
        "item must arrive in dst after import"
    );
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
        CreateArgs {
            title: "Dependent Task".to_string(),
            dependencies: vec![dep_id.clone()],
            ..Default::default()
        },
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
        CreateArgs {
            title: "Old Title".to_string(),
            description: "Body text to keep.".to_string(),
            ..Default::default()
        },
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

// ── next ─────────────────────────────────────────────────────────────────────

#[test]
fn next_skips_item_whose_blocker_is_open() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();

    // "Critical" has priority 0 (highest) but will be blocked by "Blocker" (P1).
    // Without the fix, next returns "Critical" because it has lowest priority value.
    // With the fix, next returns "Blocker" because "Critical"'s blocker is still open.
    let id_blocker = create_task(&d, "Blocker");
    commands::update::run(
        &d,
        &id_blocker,
        UpdateArgs {
            priority: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    let id_critical = create_task(&d, "Critical");
    commands::update::run(
        &d,
        &id_critical,
        UpdateArgs {
            priority: Some(0),
            ..Default::default()
        },
    )
    .unwrap();
    commands::link::run(&d, &id_blocker, "blocks", &[id_critical.clone()], false).unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "next"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "crumbs next failed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Blocker"),
        "next should return the open blocker, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("Critical"),
        "next must not return an item with an open blocker, got:\n{stdout}"
    );
}

#[test]
fn next_returns_item_once_blocker_is_closed() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();

    let id_blocker = create_task(&d, "Now Closed");
    let id_target = create_task(&d, "Ready To Do");
    commands::update::run(
        &d,
        &id_target,
        UpdateArgs {
            priority: Some(0),
            ..Default::default()
        },
    )
    .unwrap();
    commands::link::run(&d, &id_blocker, "blocks", &[id_target.clone()], false).unwrap();
    commands::close::run(&d, &id_blocker, Some("done".to_string())).unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "next"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "crumbs next failed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Ready To Do"),
        "next should surface the target once its blocker is closed, got:\n{stdout}"
    );
}

// ── list --tag AND semantics ──────────────────────────────────────────────────

#[test]
fn list_tag_filter_comma_is_and_semantics() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Both Tags".to_string(),
            tags: vec!["alpha".to_string(), "beta".to_string()],
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Only Alpha".to_string(),
            tags: vec!["alpha".to_string()],
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Neither".to_string(),
            tags: vec!["gamma".to_string()],
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "list", "--tag", "alpha,beta"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "crumbs list failed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Both Tags"),
        "AND filter should include item with both tags, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("Only Alpha"),
        "AND filter must exclude item missing one tag, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("Neither"),
        "AND filter must exclude item with neither tag, got:\n{stdout}"
    );
}

#[test]
fn list_tag_filter_single_tag_unchanged() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Has Tag".to_string(),
            tags: vec!["project/auth".to_string()],
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "No Tag".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "list", "--tag", "auth"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "crumbs list failed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Has Tag"),
        "single tag filter should match via substring, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("No Tag"),
        "single tag filter must exclude untagged items, got:\n{stdout}"
    );
}

// ── Emoji shortcode expansion ─────────────────────────────────────────────────

#[test]
fn emoji_shortcodes_expanded_on_create() {
    let dir = tempdir().unwrap();
    commands::create::run(
        dir.path(),
        CreateArgs {
            title: "Emoji test".to_string(),
            description: ":tada:".to_string(),
            ..Default::default()
        },
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
        CreateArgs {
            title: "A bug".to_string(),
            item_type: ItemType::Bug,
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        dir.path(),
        CreateArgs {
            title: "A feature".to_string(),
            item_type: ItemType::Feature,
            ..Default::default()
        },
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    let sorted = commands::list::sort_items(items, SortKey::Type);
    assert_eq!(sorted[0].1.item_type, ItemType::Bug);
    assert_eq!(sorted[1].1.item_type, ItemType::Feature);
}

#[test]
fn sort_by_due_undated_items_sort_last() {
    let dir = tempdir().unwrap();
    let id_no_due = create_task(dir.path(), "No due date");
    let id_due = create_task(dir.path(), "Has due date");
    commands::update::run(
        dir.path(),
        &id_due,
        UpdateArgs {
            due: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            ..Default::default()
        },
    )
    .unwrap();
    let items = store::load_all(dir.path()).unwrap();
    let sorted = commands::list::sort_items(items, SortKey::Due);
    // Dated item must come first; undated item must sort to the end.
    assert_eq!(sorted[0].1.id, id_due, "dated item should sort first");
    assert_eq!(sorted[1].1.id, id_no_due, "undated item should sort last");
}

#[test]
fn sort_key_from_str_error_on_unknown_field() {
    let result = "bogus".parse::<SortKey>();
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("unknown sort key"),
        "error should mention unknown key: {msg}"
    );
    assert!(msg.contains("id"), "error should list valid keys: {msg}");
}

#[test]
fn sort_key_value_enum_has_all_variants() {
    use clap::ValueEnum as _;
    let names: Vec<String> = SortKey::value_variants()
        .iter()
        .map(|v| v.to_possible_value().unwrap().get_name().to_owned())
        .collect();
    assert_eq!(
        names,
        vec![
            "id", "priority", "status", "title", "type", "due", "created", "updated"
        ],
    );
}

#[test]
fn reopen_moves_closed_reason_to_body() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id = create_task(&d, "Reopen me");

    // Close with a reason.
    commands::close::run(&d, &id, Some("fixed in PR #1".to_string())).unwrap();
    let (_, closed) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(closed.status, Status::Closed);
    assert_eq!(closed.closed_reason, "fixed in PR #1");

    // Reopen.
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let (path, reopened) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(reopened.status, Status::Open);
    // closed_reason must be cleared from frontmatter.
    assert!(
        reopened.closed_reason.is_empty(),
        "closed_reason should be empty after reopen"
    );
    // The old reason must appear in the body.
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        raw.contains("fixed in PR #1"),
        "closed_reason should be preserved in body"
    );
    assert!(
        raw.contains("Reopened"),
        "body should note that the item was reopened"
    );
}

#[test]
fn reopen_without_closed_reason_leaves_body_unchanged() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id = create_task(&d, "No reason close");

    // Close without a reason.
    commands::close::run(&d, &id, None).unwrap();

    // Reopen.
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let (path, reopened) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(reopened.status, Status::Open);
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        !raw.contains("Reopened"),
        "body should not gain a reopen note when closed_reason was empty"
    );
}

#[test]
fn reopen_with_existing_body_appends_note_after_content() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id = create_task(&d, "Body close");

    // Add body content, then close with a reason.
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            message: Some("original body text".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    commands::close::run(&d, &id, Some("done".to_string())).unwrap();

    // Reopen.
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let (path, reopened) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(reopened.status, Status::Open);
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        raw.contains("original body text"),
        "existing body should be preserved"
    );
    // Reopen note must come after existing content.
    let body_pos = raw.find("original body text").unwrap();
    let note_pos = raw.find("Reopened").unwrap();
    assert!(
        note_pos > body_pos,
        "reopen note should appear after existing body content"
    );
}

#[test]
fn reopen_with_simultaneous_message_includes_both() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id = create_task(&d, "Message reopen");

    commands::close::run(&d, &id, Some("not needed yet".to_string())).unwrap();

    // Reopen with a replacement message.
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            status: Some("open".to_string()),
            message: Some("back in scope".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let (path, reopened) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(reopened.status, Status::Open);
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        raw.contains("back in scope"),
        "new message should appear in body"
    );
    assert!(
        raw.contains("Reopened"),
        "reopen note should also appear in body"
    );
    assert!(
        raw.contains("not needed yet"),
        "original closed_reason should be in reopen note"
    );
}

#[test]
fn reopen_with_append_note_comes_after_appended_text() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id = create_task(&d, "Append reopen");

    commands::close::run(&d, &id, Some("blocked by infra".to_string())).unwrap();

    // Reopen with --append.
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            status: Some("open".to_string()),
            message: Some("picking this back up".to_string()),
            append: true,
            ..Default::default()
        },
    )
    .unwrap();

    let (path, reopened) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(reopened.status, Status::Open);
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        raw.contains("picking this back up"),
        "appended text should appear in body"
    );
    assert!(
        raw.contains("Reopened"),
        "reopen note should appear in body"
    );
    // Appended text must precede the reopen note.
    let append_pos = raw.find("picking this back up").unwrap();
    let note_pos = raw.find("Reopened").unwrap();
    assert!(
        append_pos < note_pos,
        "appended text should come before the reopen note"
    );
}
