# Remove `depends` Field Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the `dependencies` field from `Item` entirely, auto-promoting any existing `depends:` YAML values to `blocked_by`/`blocks` (bidirectionally) on the first read, so stores self-migrate lazily without any explicit migration command.

**Architecture:** `dependencies` stays in the struct as a deserialize-only field (`skip_serializing`) so existing `.md` files parse without error. `store::migrate_depends` is called from `store::load_all` for any item that still has populated `dependencies`; it extends `blocked_by` on the item, extends `blocks` on each referenced item, and rewrites both files via `rewrite_frontmatter`. After migration the field is cleared; serialization never emits it. All CLI flags (`--depends`), Tauri commands (`update_dependencies`), GUI panels, tests, and exports that reference `dependencies` are removed.

**Tech Stack:** Rust (`serde_yaml_ng`, `anyhow`), Tauri 2, vanilla JS. Tests via `cargo nextest`. Worktree: `.worktrees/remove-depends`.

---

## File Map

| File | Change |
|---|---|
| `crumbs/src/item.rs` | `dependencies` → `skip_serializing`, `skip_serializing_if` removed |
| `crumbs/src/store.rs` | Add `migrate_depends(dir, path, item, all_items)`; call from `load_all` |
| `crumbs/src/commands/create.rs` | Remove `dependencies` field from `CreateArgs`; remove from `Item` construction |
| `crumbs/src/commands/update.rs` | Remove `dependencies` field from `UpdateArgs`; remove from `has_any_mutation`; remove apply branch |
| `crumbs/src/commands/show.rs` | Remove `Deps:` display line |
| `crumbs/src/commands/export.rs` | Remove `dependencies` column from CSV header + row; update `file_import.rs` CSV reader; update test |
| `crumbs/src/commands/file_import.rs` | Remove `dependencies` from CSV struct construction |
| `crumbs/src/commands/batch_create.rs` | Remove `dependencies` from `BatchSpec` struct |
| `crumbs/src/commands/row.rs` | Remove `dependencies: vec![]` from test fixture |
| `crumbs/src/main.rs` | Remove `--depends` CLI flag from `create` and `update` subcommands |
| `crumbs/tests/commands.rs` | Update `create_with_dependencies_stores_them` → verify migration; remove `update_replaces_dependencies` test; update CSV export test |
| `crumbs-gui/src-tauri/src/commands.rs` | Remove `update_dependencies` command |
| `crumbs-gui/src-tauri/src/main.rs` | Remove `update_dependencies` from `.invoke_handler` |
| `crumbs-gui/main.js` | Remove `depsInput`, `depsRow`, `doUpdateDependencies` function, `loadedDeps` variable, `navChips(item.dependencies)` call |

---

## Task 1: Make `dependencies` deserialize-only and add migration in `store::load_all`

**Files:**
- Modify: `crumbs/src/item.rs`
- Modify: `crumbs/src/store.rs`
- Test: `crumbs/tests/commands.rs`

This is the core behaviour change. Everything else is cleanup.

- [ ] **Step 1: Write a failing integration test**

In `crumbs/tests/commands.rs`, find `create_with_dependencies_stores_them` and replace it with a migration test:

```rust
#[test]
fn depends_field_is_promoted_to_blocked_by_on_load() {
    let dir = tempdir().unwrap();
    let store = dir.path().join(".crumbs");
    std::fs::create_dir_all(&store).unwrap();
    crumbs::store_config::StoreConfig::default().save(&store).unwrap();

    // Write two items manually — one with a `depends` key pointing at the other.
    let blocker_raw = "---\nid: cr-aaa\ntitle: Blocker\nstatus: open\ntype: task\npriority: 3\ntags: []\ncreated: '2026-01-01'\nupdated: '2026-01-01'\nclosed_reason: ''\nblocks: []\nblocked_by: []\nphase: ''\nresolution: ''\n---\n\n# Blocker\n";
    let blocked_raw = "---\nid: cr-bbb\ntitle: Blocked\nstatus: open\ntype: task\npriority: 3\ntags: []\ncreated: '2026-01-01'\nupdated: '2026-01-01'\nclosed_reason: ''\ndependencies:\n- cr-aaa\nblocks: []\nblocked_by: []\nphase: ''\nresolution: ''\n---\n\n# Blocked\n";
    // Name files so the blocker (cr-aaa) is loaded before the blocked item
    // (cr-bbb) — load_all sorts by id, and "cr-aaa" < "cr-bbb".
    std::fs::write(store.join("aaa-blocker.md"), blocker_raw).unwrap();
    std::fs::write(store.join("bbb-blocked.md"), blocked_raw).unwrap();

    // load_all should trigger migration
    let items = crumbs::store::load_all(&store).unwrap();
    let blocker = items.iter().find(|(_, i)| i.id == "cr-aaa").map(|(_, i)| i).unwrap();
    let blocked  = items.iter().find(|(_, i)| i.id == "cr-bbb").map(|(_, i)| i).unwrap();

    // blocked_by on the item that had `depends`
    assert!(blocked.blocked_by.contains(&"cr-aaa".to_string()),
        "blocked_by should contain cr-aaa after migration");
    // dependencies should be cleared
    assert!(blocked.dependencies.is_empty(),
        "dependencies should be empty after migration");
    // blocks on the referenced item updated bidirectionally
    assert!(blocker.blocks.contains(&"cr-bbb".to_string()),
        "blocker.blocks should contain cr-bbb after migration");

    // Files on disk should reflect the migration (no more `depends:` key)
    let blocker_disk = std::fs::read_to_string(store.join("blocker.md")).unwrap();
    let blocked_disk = std::fs::read_to_string(store.join("blocked.md")).unwrap();
    assert!(!blocked_disk.contains("dependencies:"),
        "blocked.md should no longer have a dependencies key");
    assert!(blocker_disk.contains("- cr-bbb"),
        "blocker.md blocks list should include cr-bbb");
}
```

- [ ] **Step 2: Run test to confirm it fails**

```bash
cargo nextest run -p crumbs \
  --manifest-path /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/Cargo.toml \
  --test commands depends_field_is_promoted_to_blocked_by_on_load
```

Expected: FAIL — `blocked.dependencies` is not empty because migration doesn't exist yet.

- [ ] **Step 3: Change `dependencies` in `Item` to deserialize-only**

In `crumbs/src/item.rs`, change line 93 from:

```rust
    #[serde(default)]
    pub dependencies: Vec<String>,
```

to:

```rust
    /// Kept for **deserialisation only** — migrated to `blocked_by`/`blocks`
    /// on first load; never serialised back to disk.
    #[serde(default, skip_serializing)]
    pub dependencies: Vec<String>,
```

- [ ] **Step 4: Add `migrate_depends` to `store.rs`**

In `crumbs/src/store.rs`, add this function before `load_all`:

```rust
/// Promote a legacy `depends` list to bidirectional `blocked_by`/`blocks`
/// links and rewrite both sides to disk.
///
/// Called by [`load_all`] for any item that still carries a non-empty
/// `dependencies` vec. After this function returns the item's
/// `dependencies` is empty and `blocked_by` is extended; each referenced
/// item's `blocks` list is extended and its file is rewritten atomically.
///
/// Unknown dependency IDs (items not found in `dir`) are silently ignored
/// so that cross-store or deleted references do not block migration.
///
/// # Errors
///
/// Returns an error if reading or rewriting any item file fails.
fn migrate_depends(
    dir: &Path,
    path: &Path,
    item: &mut Item,
    all: &[(PathBuf, Item)],
) -> Result<()> {
    let ids = std::mem::take(&mut item.dependencies);
    for dep_id in &ids {
        // Extend this item's blocked_by (dedup)
        if !item.blocked_by.contains(dep_id) {
            item.blocked_by.push(dep_id.clone());
        }
        // Find the referenced item in the already-loaded snapshot and extend
        // its `blocks` list, then rewrite that file.
        if let Some((dep_path, dep_item)) = all
            .iter()
            .find(|(_, i)| i.id.eq_ignore_ascii_case(dep_id))
        {
            let mut dep = dep_item.clone();
            if !dep.blocks.contains(&item.id) {
                dep.blocks.push(item.id.clone());
                rewrite_frontmatter(dep_path, &dep)?;
            }
        }
    }
    // Rewrite this item without the depends field
    rewrite_frontmatter(path, item)?;
    Ok(())
}
```

- [ ] **Step 5: Call `migrate_depends` from `load_all` — two-pass**

`migrate_depends` needs a complete snapshot of the store so it can find referenced items regardless of filesystem readdir order. Do migration as a second pass *after* all items are loaded.

In `store::load_all`, add a migration pass just before the final sort:

```rust
// After the for-loop that populates `items`, and before `items.sort_by(...)`:

// --- depends migration (lazy, one-time per item) ---
// Collect indices of items that still carry the legacy `dependencies` field.
let to_migrate: Vec<usize> = items
    .iter()
    .enumerate()
    .filter(|(_, (_, item))| !item.dependencies.is_empty())
    .map(|(i, _)| i)
    .collect();

if !to_migrate.is_empty() {
    // Clone the full snapshot once so migrate_depends can look up any item.
    let snapshot: Vec<(PathBuf, Item)> = items.clone();
    for idx in to_migrate {
        let (path, item) = &mut items[idx];
        if let Err(e) = migrate_depends(dir, path, item, &snapshot) {
            eprintln!("warning: depends migration failed for {}: {e}", path.display());
        }
    }
}
// --- end migration ---

items.sort_by(|a, b| a.1.id.cmp(&b.1.id));
```

- [ ] **Step 6: Run the new test and all tests**

```bash
cargo nextest run -p crumbs \
  --manifest-path /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/Cargo.toml
```

Expected: all tests pass (the new test should now pass; note `create_with_dependencies_stores_them` and `update_replaces_dependencies` will still pass because `dependencies` still deserialises and the CLI flags still exist — those are removed in later tasks).

- [ ] **Step 7: Commit**

```bash
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends mit es
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  add crumbs/src/item.rs crumbs/src/store.rs crumbs/tests/commands.rs
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  commit -m "Feat: auto-migrate depends to blocked_by/blocks on load"
```

---

## Task 2: Remove `dependencies` from CLI, `CreateArgs`, `UpdateArgs`, and commands

**Files:**
- Modify: `crumbs/src/main.rs`
- Modify: `crumbs/src/commands/create.rs`
- Modify: `crumbs/src/commands/update.rs`
- Modify: `crumbs/src/commands/batch_create.rs`
- Modify: `crumbs/src/commands/show.rs`
- Modify: `crumbs/src/commands/row.rs`

- [ ] **Step 1: Remove `dependencies` from `CreateArgs`**

In `crumbs/src/commands/create.rs`:

Remove the field from the struct:
```rust
// DELETE this field:
    pub dependencies: Vec<String>,
```

Remove from `CreateArgs::default()` equivalent (the `..Default::default()` will cover it once the field is gone) and from `Item` construction in `run`:
```rust
// DELETE this line in the Item { ... } constructor:
        dependencies: args.dependencies,
```

- [ ] **Step 2: Remove `dependencies` from `UpdateArgs`**

In `crumbs/src/commands/update.rs`:

Remove the field:
```rust
// DELETE:
    pub dependencies: Option<Vec<String>>,
```

Remove from `has_any_mutation`:
```rust
// DELETE:
            || self.dependencies.is_some()
```

Remove the apply branch in `run` (~line 155):
```rust
// DELETE:
    if let Some(d) = &args.dependencies {
        item.dependencies.clone_from(d);
    }
```

- [ ] **Step 3: Remove `dependencies` from `BatchSpec` in `batch_create.rs`**

In `crumbs/src/commands/batch_create.rs`:

Remove from the `BatchSpec` struct:
```rust
// DELETE:
    pub dependencies: Vec<String>,
```

Remove from `BatchSpec::default()` / `Default` impl:
```rust
// DELETE:
            dependencies: Vec::new(),
```

Remove from the `CreateArgs { ... }` construction:
```rust
// DELETE:
            dependencies: spec.dependencies,
```

- [ ] **Step 4: Remove `--depends` from CLI in `main.rs`**

In `crumbs/src/main.rs`, find the `Create` variant (around line 89) and remove:
```rust
// DELETE from Create args:
        #[arg(long, value_delimiter = ',')]
        depends: Option<String>,
```

Find the `Update` variant (around line 117) and remove:
```rust
// DELETE from Update args:
        #[arg(long, value_delimiter = ',')]
        depends: Option<String>,
```

Remove the `depends` field from the `CreateArgs { ... }` construction (around line 383):
```rust
// DELETE:
                    dependencies: depends.map(|d| split_csv(&d)).unwrap_or_default(),
```

Remove the `depends` field from the `UpdateArgs { ... }` construction (around line 483):
```rust
// DELETE:
                dependencies: depends.map(|d| split_csv(&d)),
```

- [ ] **Step 5: Remove `Deps:` line from `show.rs`**

In `crumbs/src/commands/show.rs`, delete:
```rust
// DELETE:
            if !item.dependencies.is_empty() {
                println!("  Deps:     {}", item.dependencies.join(", "));
            }
```

- [ ] **Step 6: Remove `dependencies: vec![]` from `row.rs` test fixture**

In `crumbs/src/commands/row.rs`, delete from the `Item { ... }` literal:
```rust
// DELETE:
            dependencies: vec![],
```

- [ ] **Step 7: Check compilation**

```bash
cargo check -p crumbs \
  --manifest-path /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/Cargo.toml
```

Fix any remaining compiler errors. Do not proceed until this is clean.

- [ ] **Step 8: Remove obsolete tests in `commands.rs`**

In `crumbs/tests/commands.rs`, delete the two tests that are now impossible:

```rust
// DELETE the entire test:
#[test]
fn create_with_dependencies_stores_them() { ... }

// DELETE the entire test:
#[test]
fn update_replaces_dependencies() { ... }
```

- [ ] **Step 9: Run all tests**

```bash
cargo nextest run -p crumbs \
  --manifest-path /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/Cargo.toml
```

Expected: all remaining tests pass.

- [ ] **Step 10: Commit**

```bash
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends mit es
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  add crumbs/src/main.rs \
      crumbs/src/commands/create.rs \
      crumbs/src/commands/update.rs \
      crumbs/src/commands/batch_create.rs \
      crumbs/src/commands/show.rs \
      crumbs/src/commands/row.rs \
      crumbs/tests/commands.rs
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  commit -m "Refactor: remove depends/dependencies from CLI and commands"
```

---

## Task 3: Remove `dependencies` from CSV export/import

**Files:**
- Modify: `crumbs/src/commands/export.rs`
- Modify: `crumbs/src/commands/file_import.rs`

- [ ] **Step 1: Remove `dependencies` column from CSV export**

In `crumbs/src/commands/export.rs`, remove `"dependencies"` from the header `write_record` call and `&item.dependencies.join("|")` from the data row.

The header should become:
```rust
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
```

The data row should become (remove the `&item.dependencies.join("|"),` line):
```rust
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
    &item.story_points.map(|sp| sp.to_string()).unwrap_or_default(),
    &item.resolution,
])?;
```

- [ ] **Step 2: Update the CSV export test**

In `crumbs/src/commands/export.rs`, find `export_csv_includes_blocks_and_blocked_by`. Remove the `dep_idx` lines and the assertion that checks column ordering relative to `dependencies`:

```rust
#[test]
fn export_csv_includes_blocks_and_blocked_by() {
    let csv = items_to_string(&[sample_item()], "csv").unwrap();
    let mut rdr = csv::Reader::from_reader(csv.as_bytes());
    let headers = rdr.headers().unwrap().clone();
    let cols: Vec<&str> = headers.iter().collect();

    // dependencies column must no longer exist
    assert!(!cols.contains(&"dependencies"), "dependencies column should be removed");
    assert!(cols.contains(&"blocks"), "blocks column should exist");
    assert!(cols.contains(&"blocked_by"), "blocked_by column should exist");

    let row = rdr.records().next().unwrap().unwrap();
    let col = |name: &str| cols.iter().position(|c| *c == name).unwrap();
    assert_eq!(row.get(col("blocks")), Some("cr-aaa|cr-bbb"));
    assert_eq!(row.get(col("blocked_by")), Some("cr-zzz"));
}
```

Also remove `dependencies: vec!["cr-dep".to_string()]` from `sample_item()` in that file:
```rust
// In sample_item(), DELETE:
            dependencies: vec!["cr-dep".to_string()],
```

- [ ] **Step 3: Remove `dependencies` from CSV import in `file_import.rs`**

In `crumbs/src/commands/file_import.rs`, delete from the `Item { ... }` construction:
```rust
// DELETE:
            dependencies: split_pipe(col("dependencies")),
```

Since `dependencies` still deserialises (for migration), this field will default to `Vec::new()` when importing from a CSV that no longer has the column.

- [ ] **Step 4: Run all tests**

```bash
cargo nextest run -p crumbs \
  --manifest-path /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/Cargo.toml
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends mit es
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  add crumbs/src/commands/export.rs \
      crumbs/src/commands/file_import.rs
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  commit -m "Refactor: remove dependencies column from CSV export/import"
```

---

## Task 4: Remove `update_dependencies` Tauri command and GUI panel

**Files:**
- Modify: `crumbs-gui/src-tauri/src/commands.rs`
- Modify: `crumbs-gui/src-tauri/src/main.rs`
- Modify: `crumbs-gui/main.js`

- [ ] **Step 1: Delete `update_dependencies` from `commands.rs`**

In `crumbs-gui/src-tauri/src/commands.rs`, delete the entire function:

```rust
// DELETE entirely (~lines 184-202):
/// Update an item's dependencies. Empty string clears all dependencies.
#[tauri::command]
pub fn update_dependencies(dir: String, id: String, dependencies: String) -> Result<(), String> {
    let dep_list: Vec<String> = if dependencies.is_empty() {
        vec![]
    } else {
        dependencies
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    let store = PathBuf::from(&dir);
    commands::update::run(
        &store,
        &id,
        UpdateArgs {
            dependencies: Some(dep_list),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Remove `update_dependencies` from `main.rs` invoke handler**

In `crumbs-gui/src-tauri/src/main.rs`, remove `commands::update_dependencies,` from the `.invoke_handler(tauri::generate_handler![...])` call.

- [ ] **Step 3: Check GUI backend compiles**

```bash
cargo check -p crumbs-gui \
  --manifest-path /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/crumbs-gui/src-tauri/Cargo.toml
```

Fix any errors before proceeding.

- [ ] **Step 4: Remove the Dependencies panel from `main.js`**

In `crumbs-gui/main.js`, delete the following block (lines ~979-1001). Read those lines first to confirm the exact range, then delete:

- The `const depsInput = ...` element creation
- The `depsInput.style.cssText = ...` line
- The `let loadedDeps = ...` line
- The three `depsInput.addEventListener(...)` calls (`focus`, `blur`, `keydown`)
- The `const depsRow = propRow('Depends on', '');` line
- The `depsRow.appendChild(depsInput);` line
- The `if ((item.dependencies ?? []).length > 0) { depsRow.appendChild(navChips(item.dependencies)); }` block

Also delete the `doUpdateDependencies` function (~lines 1557-1561):

```js
// DELETE entirely:
async function doUpdateDependencies(id, dependencies) {
  clearError();
  try {
    await invoke('update_dependencies', { dir: storeDir, id, dependencies });
  } catch (e) { showError(`Update dependencies failed: ${e}`); }
}
```

- [ ] **Step 5: Check for any remaining `dependencies` references in `main.js`**

```bash
grep -n "dependencies\|update_dependencies\|doUpdateDependencies\|depsInput\|depsRow\|loadedDeps" \
  /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/crumbs-gui/main.js
```

Expected: zero matches. Fix any that remain.

- [ ] **Step 6: Commit**

```bash
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends mit es
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  add crumbs-gui/src-tauri/src/commands.rs \
      crumbs-gui/src-tauri/src/main.rs \
      crumbs-gui/main.js
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  commit -m "Refactor: remove update_dependencies Tauri command and GUI panel"
```

---

## Task 5: Final lint, full test run, and MEMORY update

**Files:**
- Read: `MEMORY.md` (update Tauri commands list)

- [ ] **Step 1: Run clippy**

```bash
cargo clippy -p crumbs \
  --manifest-path /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/Cargo.toml \
  -- -D warnings
```

Fix all warnings. Common ones to expect: unused `split_csv` if it was only used for `--depends` (check `main.rs`).

- [ ] **Step 2: Run full test suite**

```bash
cargo nextest run -p crumbs \
  --manifest-path /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/Cargo.toml
```

Expected: all tests pass.

- [ ] **Step 3: Verify no `dependencies` references remain in source**

```bash
grep -rn "\bdependencies\b\|update_dependencies\|--depends" \
  /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/crumbs/src/ \
  /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/crumbs-gui/src-tauri/src/ \
  /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends/crumbs-gui/main.js \
  --include="*.rs" --include="*.js"
```

Expected: only the `#[serde(default, skip_serializing)]` field declaration in `item.rs` and the migration code in `store.rs` remain. Anything else is a bug.

- [ ] **Step 4: Remove `update_dependencies` from MEMORY.md Tauri commands list**

In `/Users/evensolberg/.claude/projects/-Volumes-SSD-Source-Rust-crumbs/memory/MEMORY.md`, find the Tauri commands section and remove `update_dependencies` from the list. Also update the CLI flags section to remove `--depends` references.

- [ ] **Step 5: Commit**

```bash
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends mit es
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  commit --allow-empty -m "Chore: verify depends removal complete" \
  || true  # only needed if nothing changed; otherwise commit real files
```

If any files were changed in steps 1-4, stage and commit them:

```bash
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  add -p  # review and stage only relevant files
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends mit es
git -C /Volumes/SSD/Source/Rust/crumbs/.worktrees/remove-depends \
  commit -m "Chore: final cleanup after depends removal"
```
