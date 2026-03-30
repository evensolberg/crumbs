# crumbs

Flat-folder Markdown task tracker. Hybrid binary+library crate (`crumbs/`) with a Tauri GUI (`crumbs-gui/`).

## Build & test commands

| Task | Command |
|------|---------|
| Check for errors | `cargo check -p crumbs` (or `cargo lcheck -p crumbs` with cargo-limit) |
| Run tests | `cargo nextest run -p crumbs` |
| Run tests with output | `cargo nextest run -p crumbs --no-capture` |
| Build debug | `cargo build -p crumbs` |
| Build release | `cargo install --path crumbs` |
| Build GUI (dev) | `just gui-dev` |
| Build GUI (release) | `just gui-install` |

**Prefer `cargo nextest` over `cargo test`. Prefer `cargo check` over building for quick error checks.**

Common `just` aliases: `just check`, `just test` (`t`), `just testp` (`tp`), `just build` (`b`), `just release` (`r`).

## Architecture

- `crumbs/src/lib.rs` — exports all modules
- `crumbs/src/main.rs` — clap CLI; dispatches to commands
- `crumbs/src/item.rs` — `Item` struct, `Status`/`ItemType` enums, `blocks`/`blocked_by` fields
- `crumbs/src/store.rs` — read/write `.md` files, CSV index; `find_by_id` is case-insensitive
- `crumbs/src/store_config.rs` — per-store `config.toml` (prefix)
- `crumbs/src/commands/` — one file per subcommand: `create`, `update`, `close`, `delete`, `list`, `show`, `search`, `reindex`, `init`, `edit`, `stats`, `next`, `link`, `export`, `block`, `defer`, `move_`, `import`, `clean`, `start`, `stop`
- `crumbs/tests/commands.rs` — integration tests (79 tests)
- `crumbs-gui/src-tauri/src/commands.rs` — Tauri command handlers wrapping library functions
- `crumbs-gui/main.js` — vanilla JS frontend (no framework)

## Feature parity

CLI and GUI should have equivalent functionality wherever it makes sense. When adding a feature to one version, consider whether the other needs a counterpart:

- **CLI-only exceptions**: shell completions, raw file editing (`edit`), piped export to stdout, `--append` flag — these have no meaningful GUI analog.
- **GUI-only exceptions**: drag-and-drop store switching, column picker, markdown preview, inline new-blocker creation — these have no meaningful CLI analog.
- Everything else (create, update, defer, search, export, stats, next, reindex, blocking relationships) should work in both. When in doubt, implement both.

## Key conventions

- YAML frontmatter via `serde_yaml_ng`; body text lives in the Markdown body, **not** frontmatter
- `description` field: `#[serde(default, skip_serializing_if = "String::is_empty")]` for JSON (GUI); every write path clears `item.description` before calling `serde_yaml_ng::to_string` so it never leaks into frontmatter
- `index.csv` is a read cache rebuilt after every write (`store::reindex`)
- ID format: `{prefix}-{3-char alphanumeric}`; case-insensitive lookup everywhere
- `update::run` takes `UpdateArgs` struct (not positional args) — use `..Default::default()` for unset fields
- `blocks`/`blocked_by` are updated atomically on both sides via `link` or `block` commands

## Workflow

When starting work on a crumbs item, run `crumbs start <id> [-m 'comment']`. When the work is done, run `crumbs stop <id> [-m 'comment']`. The `-m` flag is optional and can go anywhere relative to the ID. This keeps an accurate time log and change history in the item body — useful for traceability and retrospectives.

Before implementing a feature or fix, attach the implementation plan to the relevant crumb item using `crumbs update <id> --message '<plan>'` (or `--append` if a description already exists). This creates a permanent historical record of what was planned and why, directly in the item file.

## Commit style

- Subject line: `Type: description` — type must be capitalized (`Feat:`, `Fix:`, `Chore:`, `Docs:`, etc.)
- Body lines ≤ 72 chars (enforced by pre-commit hook)
- Refresh author before committing: `git mit es`
- `cargo fmt` runs automatically in the pre-commit hook and re-stages reformatted files

## Crumbs File Editing

- When closing a crumb (changing status to `done`, `cancelled`, etc.), **only change the `status` field** in the YAML frontmatter. Do not add, remove, or reorder any other fields.
- Never inject fields like `description`, `updated_at`, or anything else not already present in the file.

## GUI / Frontend (Tauri + WKWebView)

- **No HTML5 drag-and-drop.** `dragover` events don't fire reliably in Tauri's WKWebView and `dataTransfer.getData()` returns empty on `drop`. Always use `mousedown`/`mousemove`/`mouseup` + `document.elementFromPoint()` for drag interactions. Give ghost elements `pointer-events: none` so `elementFromPoint` sees through them. See `startRowDrag()` in `crumbs-gui/main.js` for the reference implementation.
- **Check for duplicate declarations before committing JS changes.** WKWebView fails silently (or with a SyntaxError) on duplicate `const`/`let`/`var` declarations across script blocks. After any edit to `main.js`, scan for redeclared names before committing.
- **CSS variables must be defined before use.** Don't reference a `--var` in component styles unless it is defined in `:root` in `style.css`.

## Rust guidelines

Follow the **Pragmatic Rust Guidelines** (Microsoft, MIT licence) when writing or reviewing Rust code:

> `/Volumes/SSD/Source/Rust/pragmatic rust guidelines.txt`

Key areas relevant to this project: idiomatic API patterns, thorough docs and examples, strong types over primitives, testable APIs, and good test coverage. The project already uses `anyhow` for application-level errors per M-APP-ERROR.

## Release

- `just release` — installs to `~/.cargo/bin`, then `cargo clean`
- `just releasea` — same for `aarch64-apple-darwin`
- `just publish` — tags version, pushes to GitHub (CI builds and uploads artifacts)
- Bump version in: `crumbs/Cargo.toml`, `crumbs-gui/src-tauri/Cargo.toml`, `crumbs-gui/src-tauri/tauri.conf.json`
