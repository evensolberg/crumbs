use assert_cmd::Command;
use chrono::NaiveDate;
use predicates::str as pstr;
use tempfile::tempdir;

use crumbs::{
    commands,
    commands::batch_create::BatchCreateItem,
    commands::create::CreateArgs,
    commands::filter::{self as filter_mod, FilterArgs},
    commands::list::{ListArgs, SortKey},
    commands::update::{BulkUpdateArgs, UpdateArgs},
    item::{Item, ItemType, Status},
    store, store_config,
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

#[test]
fn close_stops_active_timer() {
    // cr-613: closing an item with a running timer must write a [stop] entry
    // and still set status to Closed.
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "Timer Close");
    commands::start::run(dir.path(), &id, None).unwrap();
    commands::close::run(dir.path(), &id, None).unwrap();

    let (path, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.status, Status::Closed);
    let raw = std::fs::read_to_string(path).unwrap();
    assert!(
        raw.contains("[stop]"),
        "close should have written a [stop] entry, got:\n{raw}"
    );
}

#[test]
fn close_without_active_timer_succeeds() {
    // cr-613: closing an item with no running timer must still succeed normally
    let dir = tempdir().unwrap();
    let id = create_task(dir.path(), "No Timer Close");
    commands::close::run(dir.path(), &id, None).unwrap();
    let (_, item) = store::find_by_id(dir.path(), &id).unwrap().unwrap();
    assert_eq!(item.status, Status::Closed);
}

#[test]
fn close_without_reason_when_not_interactive_succeeds_without_prompt() {
    // cr-by7: the CLI must not hang or prompt when stdin is not a TTY.
    // Drive the binary directly with stdin=null to exercise the main.rs
    // dispatch path (where the prompt lives).
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "No Reason Close");
    let bin = env!("CARGO_BIN_EXE_crumbs");
    let output = std::process::Command::new(bin)
        .args(["--dir", dir.path().to_str().unwrap(), "close", &id])
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "close failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let (_, item) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(item.status, Status::Closed);
    assert_eq!(item.closed_reason, "");
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

// ── move / pull ───────────────────────────────────────────────────────────────

#[test]
fn move_transfers_item_to_destination() {
    let src = tempdir().unwrap();
    let dst = tempdir().unwrap();
    // init::run returns early if the dir already exists, so point it at the
    // .crumbs subdirectory (which doesn't exist yet) so crumbs.toml is written.
    let src_store = src.path().join(".crumbs");
    let dst_store = dst.path().join(".crumbs");
    commands::init::run(&src_store, Some("src".to_string())).unwrap();
    commands::init::run(&dst_store, Some("dst".to_string())).unwrap();
    let id = create_task(&src_store, "Move Me");
    commands::move_::run(&src_store, &id, &dst_store).unwrap();
    // Item is gone from source.
    assert!(store::find_by_id(&src_store, &id).unwrap().is_none());
    // Item appears in destination with a new ID that uses the dst prefix.
    let dst_prefix = store_config::load(&dst_store).prefix;
    let expected_prefix = format!("{dst_prefix}-");
    let items = store::load_all(&dst_store).unwrap();
    let moved = items
        .iter()
        .find(|(_, i)| i.title == "Move Me")
        .expect("moved item not found in dst store");
    assert!(
        moved.1.id.starts_with(&expected_prefix),
        "moved item ID should use dst prefix ({expected_prefix}), got: {}",
        moved.1.id
    );
}

#[test]
fn pull_direction_is_src_to_dst_not_dst_to_src() {
    // Regression test for the pull CLI dispatch bug: move_::run(&dir, &id, &src)
    // was called with src and dst swapped, moving the item the wrong way.
    // This test verifies that run(src, id, dst) moves the item FROM src TO dst,
    // not from dst to src.
    //
    // init::run returns early if the dir already exists, so point it at the
    // .crumbs subdirectory (which doesn't exist yet) so crumbs.toml is written.
    let src = tempdir().unwrap();
    let dst = tempdir().unwrap();
    let src_store = src.path().join(".crumbs");
    let dst_store = dst.path().join(".crumbs");
    commands::init::run(&src_store, Some("src".to_string())).unwrap();
    commands::init::run(&dst_store, Some("dst".to_string())).unwrap();
    let id = create_task(&src_store, "Pull Me");
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
        dst_items.iter().any(|(_, i)| i.title == "Pull Me"),
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
fn pull_cli_dispatch_moves_item_from_src_to_dst() {
    // Binary-level regression test for the CLI dispatch bug: `Command::Pull`
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
            "CLI Pull Me",
        ])
        .assert()
        .success();
    // Retrieve the generated ID.
    let src_items = store::load_all(&src_store).unwrap();
    assert_eq!(src_items.len(), 1);
    let id = src_items[0].1.id.clone();
    // Run `crumbs pull <id> --from <src_store>` targeting the dst store.
    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            dst_store.to_str().unwrap(),
            "pull",
            &id,
            "--from",
            src_store.to_str().unwrap(),
        ])
        .assert()
        .success();
    // Item must be gone from source.
    assert!(
        store::find_by_id(&src_store, &id).unwrap().is_none(),
        "item must leave src after pull"
    );
    // Item must appear in destination.
    let dst_items = store::load_all(&dst_store).unwrap();
    assert!(
        dst_items.iter().any(|(_, i)| i.title == "CLI Pull Me"),
        "item must arrive in dst after pull"
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
fn depends_field_is_promoted_to_blocked_by_on_load() {
    let dir = tempdir().unwrap();
    let store = dir.path().join(".crumbs");
    std::fs::create_dir_all(&store).unwrap();
    commands::init::run(&store, Some("cr".to_string())).unwrap();

    let blocker_raw = "---\nid: cr-aaa\ntitle: Blocker\nstatus: open\ntype: task\npriority: 3\ntags: []\ncreated: '2026-01-01'\nupdated: '2026-01-01'\nclosed_reason: ''\nblocks: []\nblocked_by: []\nphase: ''\nresolution: ''\n---\n\n# Blocker\n";
    let blocked_raw = "---\nid: cr-bbb\ntitle: Blocked\nstatus: open\ntype: task\npriority: 3\ntags: []\ncreated: '2026-01-01'\nupdated: '2026-01-01'\nclosed_reason: ''\ndependencies:\n- cr-aaa\nblocks: []\nblocked_by: []\nphase: ''\nresolution: ''\n---\n\n# Blocked\n";
    std::fs::write(store.join("aaa-blocker.md"), blocker_raw).unwrap();
    std::fs::write(store.join("bbb-blocked.md"), blocked_raw).unwrap();

    let items = crumbs::store::load_all(&store).unwrap();
    let blocker = items
        .iter()
        .find(|(_, i)| i.id == "cr-aaa")
        .map(|(_, i)| i)
        .unwrap();
    let blocked = items
        .iter()
        .find(|(_, i)| i.id == "cr-bbb")
        .map(|(_, i)| i)
        .unwrap();

    assert!(
        blocked.blocked_by.contains(&"cr-aaa".to_string()),
        "blocked_by should contain cr-aaa after migration"
    );
    assert!(
        blocked.dependencies.is_empty(),
        "dependencies should be empty after migration"
    );
    assert!(
        blocker.blocks.contains(&"cr-bbb".to_string()),
        "blocker.blocks should contain cr-bbb after migration"
    );

    let blocker_disk = std::fs::read_to_string(store.join("aaa-blocker.md")).unwrap();
    let blocked_disk = std::fs::read_to_string(store.join("bbb-blocked.md")).unwrap();
    assert!(
        !blocked_disk.contains("dependencies:"),
        "bbb-blocked.md should no longer have a dependencies key"
    );
    assert!(
        blocker_disk.contains("- cr-bbb"),
        "aaa-blocker.md blocks list should include cr-bbb"
    );
}

#[test]
fn update_dependencies_field_is_not_persisted() {
    // `dependencies` is a deserialise-only migration field; setting it via
    // UpdateArgs must not cause it to be written back to disk.
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
    assert!(
        item.dependencies.is_empty(),
        "dependencies must not be persisted to disk (migration-only field)"
    );
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

#[test]
fn search_output_shows_priority_badge() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "High Pri Bug".to_string(),
            priority: 1,
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "search", "High Pri"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "crumbs search failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("[P1]"),
        "search output must include priority badge, got:\n{stdout}"
    );
}

#[test]
fn search_output_shows_phase_badge() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Phased Item".to_string(),
            phase: "phase-1".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "search", "Phased"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "crumbs search failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("[phase-1]"),
        "search output must include phase badge, got:\n{stdout}"
    );
}

#[test]
fn search_output_column_order() {
    // icon id [Px] [phase] [type] title
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Order Check".to_string(),
            priority: 0,
            item_type: "bug".parse().unwrap(),
            phase: "alpha".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "search", "Order Check"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "crumbs search failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let p0_pos = stdout.find("[P0]").expect("must contain [P0]");
    let phase_pos = stdout.find("[alpha]").expect("must contain [alpha]");
    let type_pos = stdout.find("[bug]").expect("must contain [bug]");
    assert!(p0_pos < phase_pos, "priority must come before phase");
    assert!(phase_pos < type_pos, "phase must come before type");
}

#[test]
fn search_output_shows_timer_marker() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "Timer Item");
    commands::start::run(&d, &id, None).unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "search", "Timer Item"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "crumbs search failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('▶'),
        "search output must include timer marker for active timer, got:\n{stdout}"
    );
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
    // Initialize the store so crumbs.toml is written with prefix "cr".
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
            phase: String::new(),
            resolution: String::new(),
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
fn sort_by_phase_alphabetical_no_phase_last() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id_b = create_task(&d, "Phase B item");
    let id_a = create_task(&d, "Phase A item");
    let id_none = create_task(&d, "No phase item");
    commands::update::run(
        &d,
        &id_a,
        UpdateArgs {
            phase: Some("alpha".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    commands::update::run(
        &d,
        &id_b,
        UpdateArgs {
            phase: Some("beta".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let items = store::load_all(&d).unwrap();
    let sorted = commands::list::sort_items(items, SortKey::Phase);
    assert_eq!(sorted[0].1.id, id_a, "alpha should sort first");
    assert_eq!(sorted[1].1.id, id_b, "beta should sort second");
    assert_eq!(sorted[2].1.id, id_none, "no-phase item should sort last");
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
            "id", "priority", "status", "title", "type", "due", "created", "updated", "phase"
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

// ── phase field ───────────────────────────────────────────────────────────────

#[test]
fn create_with_phase_stores_field() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Phase Task".to_string(),
            phase: "phase-1".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    let items = store::load_all(&d).unwrap();
    let item = items
        .into_iter()
        .find(|(_, i)| i.title == "Phase Task")
        .unwrap()
        .1;
    assert_eq!(item.phase, "phase-1");
}

#[test]
fn update_phase_sets_field() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "Update Phase");
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            phase: Some("2026-Q2".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(item.phase, "2026-Q2");
}

#[test]
fn update_clear_phase_clears_value() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "Clear Phase");
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            phase: Some("phase-1".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            clear_phase: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert!(
        item.phase.is_empty(),
        "clear_phase should leave phase as empty string"
    );
}

#[test]
fn update_phase_trims_whitespace() {
    // Phase values with leading/trailing whitespace must be stored trimmed so
    // that `list --phase` can match them reliably with an exact comparison.
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "Whitespace Phase");
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            phase: Some("  phase-1  ".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(item.phase, "phase-1", "phase should be stored trimmed");
}

#[test]
fn list_phase_filter_trims_whitespace() {
    // list --phase with surrounding whitespace must still match trimmed values.
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Phase One".to_string(),
            phase: "phase-1".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "list",
            "--phase",
            "  phase-1  ",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Phase One"),
        "--phase with surrounding spaces must match trimmed stored value, got:\n{stdout}"
    );
}

#[test]
fn list_phase_filter_shows_matching_items_only() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "In Phase One".to_string(),
            phase: "phase-1".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "In Phase Two".to_string(),
            phase: "phase-2".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "No Phase".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "list", "--phase", "phase-1"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("In Phase One"),
        "phase filter should include matching item, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("In Phase Two"),
        "phase filter must exclude different phase, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("No Phase"),
        "phase filter must exclude items with no phase, got:\n{stdout}"
    );
}

#[test]
fn list_phase_filter_whitespace_only_is_no_filter() {
    // An all-whitespace --phase value must be treated as "no filter" rather
    // than silently matching items that have no phase set.
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Has Phase".to_string(),
            phase: "phase-1".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "No Phase".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "list", "--phase", "   "])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Has Phase"),
        "whitespace-only phase filter must show phased items, got:\n{stdout}"
    );
    assert!(
        stdout.contains("No Phase"),
        "whitespace-only phase filter must show unphased items too, got:\n{stdout}"
    );
}

#[test]
fn list_output_shows_phase_badge() {
    // Items with a phase set should show a [phase] badge inline between the
    // priority and type badges; items without a phase should show [ ].
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Has Phase".to_string(),
            phase: "2026-Q2".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "No Phase Item".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // The phased item line must include the inline [2026-Q2] badge before [task]
    let phased_line = stdout
        .lines()
        .find(|l| l.contains("Has Phase"))
        .unwrap_or("");
    assert!(
        phased_line.contains("[2026-Q2]"),
        "list output must show [phase] badge for items with a phase, got line:\n{phased_line}"
    );
    let phase_pos = phased_line.find("[2026-Q2]").unwrap();
    let type_pos = phased_line.find("[task]").unwrap();
    assert!(
        phase_pos < type_pos,
        "phase badge must appear before type badge, got line:\n{phased_line}"
    );

    // The no-phase item line must show a badge padded to the widest phase width.
    // "2026-Q2" is 7 chars, so the empty badge should be "[       ]" (7 spaces).
    let no_phase_line = stdout
        .lines()
        .find(|l| l.contains("No Phase Item"))
        .unwrap_or("");
    let expected_empty = format!("[{}]", " ".repeat("2026-Q2".len()));
    assert!(
        no_phase_line.contains(&expected_empty),
        "list output must show empty badge padded to max phase width, got line:\n{no_phase_line}"
    );
}

#[test]
fn list_phase_badge_padded_to_widest_phase() {
    // When items have phases of different lengths, all phase badges must be
    // padded to the width of the longest phase so columns align.
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Short Phase".to_string(),
            phase: "p1".to_string(), // 2 chars
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Long Phase".to_string(),
            phase: "2026-Q2".to_string(), // 7 chars — widest
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "No Phase".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let short_line = stdout
        .lines()
        .find(|l| l.contains("Short Phase"))
        .unwrap_or("");
    let long_line = stdout
        .lines()
        .find(|l| l.contains("Long Phase"))
        .unwrap_or("");
    let none_line = stdout
        .lines()
        .find(|l| l.contains("No Phase"))
        .unwrap_or("");

    // All badges must be padded to width 7 (longest phase)
    assert!(
        short_line.contains("[p1     ]"),
        "short phase must be padded to max width, got:\n{short_line}"
    );
    assert!(
        long_line.contains("[2026-Q2]"),
        "longest phase must be unpadded, got:\n{long_line}"
    );
    assert!(
        none_line.contains("[       ]"),
        "no-phase must be padded to max width with spaces, got:\n{none_line}"
    );
}

#[test]
fn list_phase_badge_unicode_width_padding() {
    // CJK characters have display width 2. "日本" has display width 4.
    // An ASCII phase "ab" (display width 2) must be padded by 2 spaces to match.
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "CJK Phase".to_string(),
            phase: "日本".to_string(), // display width 4
            ..Default::default()
        },
    )
    .unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "ASCII Phase".to_string(),
            phase: "ab".to_string(), // display width 2
            ..Default::default()
        },
    )
    .unwrap();

    let output = Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let cjk_line = stdout
        .lines()
        .find(|l| l.contains("CJK Phase"))
        .unwrap_or("");
    let ascii_line = stdout
        .lines()
        .find(|l| l.contains("ASCII Phase"))
        .unwrap_or("");

    // CJK phase is the widest (display width 4); no padding needed
    assert!(
        cjk_line.contains("[日本]"),
        "CJK phase badge must not be padded, got:\n{cjk_line}"
    );
    // ASCII phase display width 2 must be padded by 2 spaces to reach display width 4
    assert!(
        ascii_line.contains("[ab  ]"),
        "ASCII phase must be padded to CJK display width, got:\n{ascii_line}"
    );
}

#[test]
fn create_with_empty_phase_writes_key_to_frontmatter() {
    // Passing phase: "" (or omitting --phase) must still produce `phase:` in
    // the YAML so external tools can always grep for the key.
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    commands::create::run(
        &d,
        CreateArgs {
            title: "Empty Phase".to_string(),
            phase: String::new(),
            ..Default::default()
        },
    )
    .unwrap();
    let items = store::load_all(&d).unwrap();
    let (path, item) = items
        .into_iter()
        .find(|(_, i)| i.title == "Empty Phase")
        .unwrap();
    assert!(item.phase.is_empty(), "phase should be empty string");
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        raw.contains("phase:"),
        "phase key must be in frontmatter even when empty, got:\n{raw}"
    );
}

#[test]
fn phase_always_written_to_frontmatter() {
    // Even without a phase value, the key must appear in the raw YAML so
    // external tools can grep and bulk-edit it without touching every item.
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "No Phase Item");
    let (path, _) = store::find_by_id(&d, &id).unwrap().unwrap();
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        raw.contains("phase:"),
        "phase key must be present in frontmatter even when empty, got:\n{raw}"
    );
}

#[test]
fn phase_round_trips_through_file() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "Round Trip Phase");
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            phase: Some("2026-Q3".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(
        item.phase, "2026-Q3",
        "phase should survive a write/read round-trip"
    );
}

// ── resolution field (cr-w8z) ─────────────────────────────────────────────────

#[test]
fn update_sets_resolution() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "Resolution Item");
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            resolution: Some("github.com/evensolberg/crumbs/pull/22".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(item.resolution, "github.com/evensolberg/crumbs/pull/22");
}

#[test]
fn resolution_round_trips_through_file() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "Resolution Round Trip");
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            resolution: Some("cr-w8z".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    let (_, item) = store::find_by_id(&d, &id).unwrap().unwrap();
    assert_eq!(
        item.resolution, "cr-w8z",
        "resolution should survive write/read round-trip"
    );
}

#[test]
fn resolution_not_in_frontmatter_when_empty() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "No Resolution");
    let (path, _) = store::find_by_id(&d, &id).unwrap().unwrap();
    let raw = std::fs::read_to_string(path).unwrap();
    assert!(
        !raw.contains("resolution:"),
        "resolution key should be absent when empty, got:\n{raw}"
    );
}

// ── export (markdown) ─────────────────────────────────────────────────────────

#[test]
fn export_markdown_flat_produces_table() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    create_task(&d, "Alpha");
    let md = crumbs::commands::export::to_string(&d, "markdown").unwrap();
    assert!(md.contains("| ID |"), "missing table header");
    assert!(md.contains("Alpha"), "missing item title");
}

#[test]
fn export_markdown_grouped_by_type_has_sections() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "A Task");
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            item_type: Some("feature".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    create_task(&d, "Another Task");
    let md = crumbs::commands::export::to_string(&d, "markdown?group=type").unwrap();
    assert!(md.contains("## Feature"), "missing Feature section");
    assert!(md.contains("## Task"), "missing Task section");
}

#[test]
fn export_markdown_grouped_by_status_has_sections() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "In Progress Item");
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            status: Some("in_progress".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    create_task(&d, "Open Item");
    let md = crumbs::commands::export::to_string(&d, "markdown?group=status").unwrap();
    assert!(md.contains("## in_progress"), "missing in_progress section");
    assert!(md.contains("## open"), "missing open section");
}

#[test]
fn export_markdown_cli_group_by_type_writes_md_file() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();
    let id = create_task(&d, "A Feature");
    commands::update::run(
        &d,
        &id,
        UpdateArgs {
            item_type: Some("feature".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    create_task(&d, "A Task");

    let out_path = dir.path().join("roadmap.md");
    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "export",
            "--format",
            "markdown",
            "--group-by",
            "type",
            "--output",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("## Feature"), "missing Feature section");
    assert!(content.contains("## Task"), "missing Task section");
}

#[test]
fn export_markdown_cli_group_by_without_markdown_format_errors() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("cr".to_string())).unwrap();

    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "export",
            "--format",
            "json",
            "--group-by",
            "type",
        ])
        .assert()
        .failure()
        .stderr(pstr::contains("--group-by requires --format markdown"));
}

// ── batch create ─────────────────────────────────────────────────────────────

#[test]
fn batch_create_from_json_creates_all_items() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("bc".to_string())).unwrap();

    let items = vec![
        BatchCreateItem {
            title: "Batch Alpha".to_string(),
            ..Default::default()
        },
        BatchCreateItem {
            title: "Batch Beta".to_string(),
            item_type: ItemType::Bug,
            priority: 1,
            ..Default::default()
        },
    ];
    commands::batch_create::run(&d, items).unwrap();

    let stored = store::load_all(&d).unwrap();
    assert_eq!(stored.len(), 2);
    assert!(stored.iter().any(|(_, i)| i.title == "Batch Alpha"));
    assert!(stored.iter().any(|(_, i)| i.title == "Batch Beta"));
}

#[test]
fn batch_create_generates_unique_ids_across_batch() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("bc".to_string())).unwrap();

    let items: Vec<BatchCreateItem> = (0..10)
        .map(|i| BatchCreateItem {
            title: format!("Item {i}"),
            ..Default::default()
        })
        .collect();
    commands::batch_create::run(&d, items).unwrap();

    let stored = store::load_all(&d).unwrap();
    assert_eq!(stored.len(), 10);
    let mut ids: Vec<_> = stored.iter().map(|(_, i)| i.id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 10, "all IDs must be unique");
}

#[test]
fn batch_create_empty_vec_is_noop() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("bc".to_string())).unwrap();

    commands::batch_create::run(&d, vec![]).unwrap();

    let stored = store::load_all(&d).unwrap();
    assert!(stored.is_empty());
}

#[test]
fn batch_create_from_json_file_via_cli() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("bc".to_string())).unwrap();

    let json = r#"[{"title":"From JSON File"},{"title":"Also JSON","type":"bug"}]"#;
    let json_path = dir.path().join("items.json");
    std::fs::write(&json_path, json).unwrap();

    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "batch-create",
            "--from",
            json_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stored = store::load_all(&d).unwrap();
    assert_eq!(stored.len(), 2);
    assert!(stored.iter().any(|(_, i)| i.title == "From JSON File"));
    assert!(stored.iter().any(|(_, i)| i.title == "Also JSON"));
}

#[test]
fn batch_create_from_yaml_file_via_cli() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("bc".to_string())).unwrap();

    let yaml = "- title: From YAML File\n- title: Also YAML\n  type: feature\n";
    let yaml_path = dir.path().join("items.yaml");
    std::fs::write(&yaml_path, yaml).unwrap();

    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "batch-create",
            "--from",
            yaml_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stored = store::load_all(&d).unwrap();
    assert_eq!(stored.len(), 2);
    assert!(stored.iter().any(|(_, i)| i.title == "From YAML File"));
    assert!(stored.iter().any(|(_, i)| i.title == "Also YAML"));
}

#[test]
fn batch_create_rejects_non_fibonacci_story_points() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("bc".to_string())).unwrap();
    let items = vec![BatchCreateItem {
        title: "Bad Points".to_string(),
        story_points: Some(4), // not a Fibonacci number
        ..Default::default()
    }];
    let result = commands::batch_create::run(&d, items);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Fibonacci"));
}

#[test]
fn batch_create_infer_format_json_extension() {
    let result =
        crumbs::commands::batch_create::infer_format(std::path::Path::new("items.json"), None);
    assert_eq!(result.unwrap(), "json");
}

#[test]
fn batch_create_infer_format_yaml_extension() {
    let result =
        crumbs::commands::batch_create::infer_format(std::path::Path::new("items.yaml"), None);
    assert_eq!(result.unwrap(), "yaml");
}

#[test]
fn batch_create_infer_format_yml_extension() {
    let result =
        crumbs::commands::batch_create::infer_format(std::path::Path::new("items.yml"), None);
    assert_eq!(result.unwrap(), "yaml");
}

#[test]
fn batch_create_infer_format_explicit_overrides_extension() {
    let result = crumbs::commands::batch_create::infer_format(
        std::path::Path::new("items.json"),
        Some("yaml"),
    );
    assert_eq!(result.unwrap(), "yaml");
}

#[test]
fn batch_create_infer_format_no_extension_errors() {
    let result = crumbs::commands::batch_create::infer_format(std::path::Path::new("items"), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("use --format"));
}

#[test]
fn batch_create_unknown_extension_without_format_errors() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("bc".to_string())).unwrap();

    let path = dir.path().join("items.txt");
    std::fs::write(&path, "[]").unwrap();

    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "batch-create",
            "--from",
            path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(pstr::contains("use --format"));
}

#[test]
fn batch_create_from_stdin_json_creates_items() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("bc".to_string())).unwrap();

    let json = r#"[{"title":"Stdin JSON Item"},{"title":"Another Stdin Item"}]"#;

    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "batch-create",
            "--from",
            "-",
            "--format",
            "json",
        ])
        .write_stdin(json)
        .assert()
        .success();

    let stored = store::load_all(&d).unwrap();
    assert_eq!(stored.len(), 2);
    assert!(stored.iter().any(|(_, i)| i.title == "Stdin JSON Item"));
    assert!(stored.iter().any(|(_, i)| i.title == "Another Stdin Item"));
}

#[test]
fn batch_create_from_stdin_missing_format_errors() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("bc".to_string())).unwrap();

    Command::cargo_bin("crumbs")
        .unwrap()
        .args(["--dir", d.to_str().unwrap(), "batch-create", "--from", "-"])
        .write_stdin(r#"[{"title":"Should Fail"}]"#)
        .assert()
        .failure()
        .stderr(pstr::contains("--format"));
}

// ── file import ──────────────────────────────────────────────────────────────

#[test]
fn file_import_from_json_preserves_id_and_fields() {
    let src = tempdir().unwrap();
    let src_store = src.path().join(".crumbs");
    commands::init::run(&src_store, Some("fi".to_string())).unwrap();
    let id = create_task(&src_store, "Preserve Me");
    let (_, item) = store::find_by_id(&src_store, &id).unwrap().unwrap();

    let json = serde_json::to_string(&vec![item]).unwrap();
    let json_path = src.path().join("export.json");
    std::fs::write(&json_path, json).unwrap();

    let dst = tempdir().unwrap();
    let dst_store = dst.path().join(".crumbs");
    commands::init::run(&dst_store, Some("fi".to_string())).unwrap();

    commands::file_import::run(&dst_store, &json_path, None).unwrap();

    let stored = store::load_all(&dst_store).unwrap();
    assert_eq!(stored.len(), 1);
    let imported = &stored[0].1;
    assert_eq!(imported.id, id, "ID must be preserved");
    assert_eq!(imported.title, "Preserve Me");
}

#[test]
fn file_import_from_csv_creates_items() {
    let src = tempdir().unwrap();
    let src_store = src.path().join(".crumbs");
    commands::init::run(&src_store, Some("fi".to_string())).unwrap();
    create_task(&src_store, "CSV Import");
    let items: Vec<_> = store::load_all(&src_store)
        .unwrap()
        .into_iter()
        .map(|(_, i)| i)
        .collect();

    let csv_str = crumbs::commands::export::items_to_string(&items, "csv").unwrap();
    let csv_path = src.path().join("export.csv");
    std::fs::write(&csv_path, csv_str).unwrap();

    let dst = tempdir().unwrap();
    let dst_store = dst.path().join(".crumbs");
    commands::init::run(&dst_store, Some("fi".to_string())).unwrap();

    commands::file_import::run(&dst_store, &csv_path, None).unwrap();

    let stored = store::load_all(&dst_store).unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].1.title, "CSV Import");
}

#[test]
fn file_import_infers_format_from_json_extension() {
    let result =
        crumbs::commands::file_import::infer_format(std::path::Path::new("data.json"), None);
    assert_eq!(result.unwrap(), "json");
}

#[test]
fn file_import_infers_format_from_csv_extension() {
    let result =
        crumbs::commands::file_import::infer_format(std::path::Path::new("data.csv"), None);
    assert_eq!(result.unwrap(), "csv");
}

#[test]
fn file_import_toon_extension_errors_with_helpful_message() {
    // TOON import is not supported due to serde_toon round-trip limitations.
    let result =
        crumbs::commands::file_import::infer_format(std::path::Path::new("data.toon"), None);
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("TOON import is not supported"),
        "unexpected error: {err}"
    );
}

#[test]
fn file_import_explicit_format_overrides_extension() {
    let result =
        crumbs::commands::file_import::infer_format(std::path::Path::new("data.csv"), Some("json"));
    assert_eq!(result.unwrap(), "json");
}

#[test]
fn file_import_unknown_extension_without_format_errors() {
    let result =
        crumbs::commands::file_import::infer_format(std::path::Path::new("data.txt"), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("use --format"));
}

#[test]
fn file_import_id_conflict_errors() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("fi".to_string())).unwrap();
    let id = create_task(&d, "Original");
    let (_, item) = store::find_by_id(&d, &id).unwrap().unwrap();

    let json = serde_json::to_string(&vec![item]).unwrap();
    let json_path = dir.path().join("export.json");
    std::fs::write(&json_path, json).unwrap();

    // Importing the same item again should error on the ID conflict.
    let result = commands::file_import::run(&d, &json_path, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn file_import_toon_format_returns_unsupported_error() {
    // TOON import is unsupported: serde_toon cannot round-trip enum variants.
    // Verify the error message points users toward JSON instead.
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("fi".to_string())).unwrap();

    let toon_path = dir.path().join("export.toon");
    std::fs::write(&toon_path, "[1]: []").unwrap();

    let result = commands::file_import::run(&d, &toon_path, None);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("not supported") || msg.contains("use --format"),
        "{msg}"
    );
}

#[test]
fn file_import_duplicate_id_in_file_errors() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("fi".to_string())).unwrap();

    // Build a JSON array where the same ID appears twice.
    let id = create_task(&d, "Original");
    let (_, item) = store::find_by_id(&d, &id).unwrap().unwrap();
    let json = serde_json::to_string(&vec![item.clone(), item]).unwrap();

    // Use a fresh destination store so the ID is not already there.
    let dst = tempdir().unwrap();
    let dst_store = dst.path().join(".crumbs");
    commands::init::run(&dst_store, Some("fi".to_string())).unwrap();

    let json_path = dst.path().join("dupes.json");
    std::fs::write(&json_path, json).unwrap();

    let result = commands::file_import::run(&dst_store, &json_path, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("more than once"));
}

#[test]
fn file_import_via_cli_json() {
    let src = tempdir().unwrap();
    let src_store = src.path().join(".crumbs");
    commands::init::run(&src_store, Some("fi".to_string())).unwrap();
    let id = create_task(&src_store, "CLI JSON Import");
    let items: Vec<_> = store::load_all(&src_store)
        .unwrap()
        .into_iter()
        .map(|(_, i)| i)
        .collect();

    let json = serde_json::to_string(&items).unwrap();
    let json_path = src.path().join("export.json");
    std::fs::write(&json_path, json).unwrap();

    let dst = tempdir().unwrap();
    let dst_store = dst.path().join(".crumbs");
    commands::init::run(&dst_store, Some("fi".to_string())).unwrap();

    Command::cargo_bin("crumbs")
        .unwrap()
        .args([
            "--dir",
            dst_store.to_str().unwrap(),
            "import",
            "--file",
            json_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stored = store::load_all(&dst_store).unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].1.id, id, "ID must be preserved");
    assert_eq!(stored[0].1.title, "CLI JSON Import");
}

// ── filter::apply ─────────────────────────────────────────────────────────────

#[test]
fn filter_by_tag_returns_matching_items() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id1 = create_task(&d, "Tagged Item");
    let _id2 = create_task(&d, "Other Item");
    commands::update::run(
        &d,
        &id1,
        UpdateArgs {
            tags: Some(vec!["sprint/3".to_string()]),
            ..Default::default()
        },
    )
    .unwrap();

    let items = store::load_all(&d).unwrap();
    let filtered = filter_mod::apply(
        items,
        &FilterArgs {
            tag: Some("sprint/3".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].1.id, id1);
}

#[test]
fn filter_by_priority_returns_matching_items() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id1 = create_task(&d, "High Priority");
    let _id2 = create_task(&d, "Normal Priority");
    commands::update::run(
        &d,
        &id1,
        UpdateArgs {
            priority: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    let items = store::load_all(&d).unwrap();
    let filtered = filter_mod::apply(
        items,
        &FilterArgs {
            priority: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].1.id, id1);
}

#[test]
fn filter_by_tag_and_priority_applies_and_semantics() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    // A: tag=sprint/3, priority=1 — matches both
    let id_a = create_task(&d, "Item A");
    // B: tag=sprint/3, priority=2 — only tag matches
    let id_b = create_task(&d, "Item B");
    // C: tag=other, priority=1 — only priority matches
    let id_c = create_task(&d, "Item C");
    commands::update::run(
        &d,
        &id_a,
        UpdateArgs {
            tags: Some(vec!["sprint/3".to_string()]),
            priority: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    commands::update::run(
        &d,
        &id_b,
        UpdateArgs {
            tags: Some(vec!["sprint/3".to_string()]),
            priority: Some(2),
            ..Default::default()
        },
    )
    .unwrap();
    commands::update::run(
        &d,
        &id_c,
        UpdateArgs {
            tags: Some(vec!["other".to_string()]),
            priority: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    let items = store::load_all(&d).unwrap();
    let filtered = filter_mod::apply(
        items,
        &FilterArgs {
            tag: Some("sprint/3".to_string()),
            priority: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(filtered.len(), 1, "only item A matches both filters");
    assert_eq!(filtered[0].1.id, id_a);
}

#[test]
fn filter_invalid_status_returns_error() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    create_task(&d, "Any Item");

    let items = store::load_all(&d).unwrap();
    let result = filter_mod::apply(
        items,
        &FilterArgs {
            status: Some("not_a_real_status".to_string()),
            ..Default::default()
        },
    );

    assert!(result.is_err());
}

#[test]
fn filter_all_includes_closed_items() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id1 = create_task(&d, "Open Item");
    let id2 = create_task(&d, "Closed Item");
    commands::close::run(&d, &id2, None).unwrap();

    // Without all: closed items are hidden
    let items = store::load_all(&d).unwrap();
    let without_all = filter_mod::apply(items, &FilterArgs::default()).unwrap();
    assert_eq!(without_all.len(), 1, "closed item hidden by default");
    assert_eq!(without_all[0].1.id, id1);

    // With all: true — closed items are included
    let items = store::load_all(&d).unwrap();
    let with_all = filter_mod::apply(
        items,
        &FilterArgs {
            all: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(with_all.len(), 2, "all: true includes closed items");
    assert!(with_all.iter().any(|(_, i)| i.id == id2));
}

// ── update::run_bulk ──────────────────────────────────────────────────────────

#[test]
fn bulk_update_priority_by_tag() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id1 = create_task(&d, "Sprint Item 1");
    let id2 = create_task(&d, "Sprint Item 2");
    let id3 = create_task(&d, "Untagged Item");
    for id in &[&id1, &id2] {
        commands::update::run(
            &d,
            id,
            UpdateArgs {
                tags: Some(vec!["sprint/3".to_string()]),
                ..Default::default()
            },
        )
        .unwrap();
    }

    commands::update::run_bulk(
        &d,
        BulkUpdateArgs {
            filter: FilterArgs {
                tag: Some("sprint/3".to_string()),
                ..Default::default()
            },
            update: UpdateArgs {
                priority: Some(1),
                ..Default::default()
            },
            dry_run: false,
        },
    )
    .unwrap();

    let items = store::load_all(&d).unwrap();
    let get = |id: &str| {
        items
            .iter()
            .find(|(_, i)| i.id == id)
            .map(|(_, i)| i.priority)
            .unwrap()
    };
    assert_eq!(get(&id1), 1, "sprint item should have priority 1");
    assert_eq!(get(&id2), 1, "sprint item should have priority 1");
    assert_ne!(get(&id3), 1, "untagged item should be unchanged");
}

#[test]
fn bulk_update_dry_run_does_not_mutate() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id1 = create_task(&d, "Tagged Item");
    commands::update::run(
        &d,
        &id1,
        UpdateArgs {
            tags: Some(vec!["sprint/3".to_string()]),
            priority: Some(5),
            ..Default::default()
        },
    )
    .unwrap();

    commands::update::run_bulk(
        &d,
        BulkUpdateArgs {
            filter: FilterArgs {
                tag: Some("sprint/3".to_string()),
                ..Default::default()
            },
            update: UpdateArgs {
                priority: Some(1),
                ..Default::default()
            },
            dry_run: true,
        },
    )
    .unwrap();

    let (_, item) = store::find_by_id(&d, &id1).unwrap().unwrap();
    assert_eq!(item.priority, 5, "dry run must not mutate the item");
}

#[test]
fn bulk_update_no_matches_returns_ok() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    create_task(&d, "Some Item");

    let result = commands::update::run_bulk(
        &d,
        BulkUpdateArgs {
            filter: FilterArgs {
                tag: Some("nonexistent-tag".to_string()),
                ..Default::default()
            },
            update: UpdateArgs {
                priority: Some(1),
                ..Default::default()
            },
            dry_run: false,
        },
    );

    assert!(result.is_ok());
}

// ── close::run_bulk ───────────────────────────────────────────────────────────

#[test]
fn bulk_close_by_tag() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id1 = create_task(&d, "Done Item 1");
    let id2 = create_task(&d, "Done Item 2");
    let id3 = create_task(&d, "Keep Open");
    for id in &[&id1, &id2] {
        commands::update::run(
            &d,
            id,
            UpdateArgs {
                tags: Some(vec!["done".to_string()]),
                ..Default::default()
            },
        )
        .unwrap();
    }

    commands::close::run_bulk(
        &d,
        FilterArgs {
            tag: Some("done".to_string()),
            ..Default::default()
        },
        None,
        false,
    )
    .unwrap();

    let (_, i1) = store::find_by_id(&d, &id1).unwrap().unwrap();
    let (_, i2) = store::find_by_id(&d, &id2).unwrap().unwrap();
    let (_, i3) = store::find_by_id(&d, &id3).unwrap().unwrap();
    assert_eq!(i1.status, Status::Closed, "tagged item should be closed");
    assert_eq!(i2.status, Status::Closed, "tagged item should be closed");
    assert_eq!(i3.status, Status::Open, "untagged item should remain open");
}

#[test]
fn bulk_close_dry_run_does_not_mutate() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id1 = create_task(&d, "Tagged Item");
    commands::update::run(
        &d,
        &id1,
        UpdateArgs {
            tags: Some(vec!["done".to_string()]),
            ..Default::default()
        },
    )
    .unwrap();

    commands::close::run_bulk(
        &d,
        FilterArgs {
            tag: Some("done".to_string()),
            ..Default::default()
        },
        None,
        true, // dry_run
    )
    .unwrap();

    let (_, item) = store::find_by_id(&d, &id1).unwrap().unwrap();
    assert_eq!(item.status, Status::Open, "dry run must not close the item");
}

#[test]
fn bulk_close_skips_already_closed_items() {
    let dir = tempdir().unwrap();
    let d = dir.path().join(".crumbs");
    commands::init::run(&d, Some("ts".to_string())).unwrap();
    let id1 = create_task(&d, "Already Closed");
    let id2 = create_task(&d, "To Be Closed");
    for id in &[&id1, &id2] {
        commands::update::run(
            &d,
            id,
            UpdateArgs {
                tags: Some(vec!["done".to_string()]),
                ..Default::default()
            },
        )
        .unwrap();
    }
    // Close id1 before running bulk close
    commands::close::run(&d, &id1, None).unwrap();

    // Now run bulk close: should only close id2
    commands::close::run_bulk(
        &d,
        FilterArgs {
            tag: Some("done".to_string()),
            ..Default::default()
        },
        None,
        false,
    )
    .unwrap();

    let (_, i1) = store::find_by_id(&d, &id1).unwrap().unwrap();
    let (_, i2) = store::find_by_id(&d, &id2).unwrap().unwrap();
    // Both are closed, but id1 was already closed (not double-closed, no error)
    assert_eq!(i1.status, Status::Closed);
    assert_eq!(i2.status, Status::Closed);
}
