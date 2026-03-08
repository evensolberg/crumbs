# Changelog

All notable changes to this project will be documented in this file.

## [0.12.0] - 2026-03-08

### Feat

- `crumbs start <id> [-m comment]` — append `[start]` entry, set status to `in_progress`
- `crumbs stop <id> [-m comment]` — append `[stop]` entry with elapsed time
- `crumbs show` now prints `Total tracked: Xh Ym Zs` when stop entries exist
- GUI: ▶ Timer / ■ Stop toolbar button with comment modal

### Fix

- Description field was leaking into YAML frontmatter on `close`, `defer`,
  `block`, `link`, and `update`; cleared before serialization at all write sites

### Docs

- README, SKILL.md, CLAUDE.md updated with time-tracking docs and workflow note

## [0.11.5] - 2026-03-08

### Feat

- `show` accepts multiple IDs; blank line between each (`crumbs show id1 id2`)
- `list --verbose` prints first two body lines beneath each item
- `update --append 'text'` appends to body with `[YYYY-MM-DD]` prefix
- `defer --until <date>` sets wake-up date; `next` skips future-deferred items
- `--message` now allows values starting with `-` (allow_hyphen_values)
- GUI: status strip with live item counts and colored status badges
- GUI: full-text search bar (debounced, hits backend)
- GUI: drag-and-drop rows to move items between stores
- GUI: defer modal with optional "until" date picker
- GUI: export modal (JSON / CSV / TOON via save dialog)
- GUI: edit dependencies inline in detail pane
- GUI: Next and Reindex buttons

### Fix

- GUI: column resize now moves only the dragged column; others stay fixed
- GUI: no text selection when dragging a column divider vertically
- `update::run` refactored to `UpdateArgs` struct (replaces 14 positional args)

## [0.9.1] - 2026-03-08

### Chore

- Add recent crumbs task items

### Feat

- Blocked-by selector modal in GUI (v0.9.0)

### Fix

- Description leaks into frontmatter; --dir ignores .crumbs (v0.9.1)

## [0.8.2] - 2026-03-07

### Chore

- Rename release job to 'Publish release'; attach crumbs.skill
- Update publish recipe; add dist recipe for local builds

### Docs

- Update README with GUI overview, screenshot, and table fixes
- Update SKILL.md with platform global store paths and story_points

### Feat

- Add Tauri GUI, restructure as workspace (v0.8.2)

### Fix

- Add missing title arg to update::run test calls
- Install cargo-tauri in GUI CI job before building
- Install correct Rust target per GUI matrix entry
- Add icon.ico required for Windows GUI build
- Add icon.ico to tauri.conf.json bundle icons for Windows build

## [0.6.3] - 2026-03-06

### Fix

- Hide --dir/--global from -h; visible in --help (v0.6.3)

## [0.6.2] - 2026-03-06

### Fix

- Use 'global' keyword for --to/--from; add skills/SKILL.md (v0.6.2)

## [0.6.1] - 2026-03-06

### Fix

- Make --to/--to-global and --from/--from-global mutually required
- Enforce --to/--from flags at parse time, bump to v0.6.1

## [0.6.0] - 2026-03-06

### Chore

- Close completed crumbs, update changelog, reformat main.rs

### Feat

- Add blocked/deferred statuses and block/defer/move/import (v0.6.0)

## [0.5.4] - 2026-03-06

### Fix

- Robustness and security improvements (v0.5.4)

## [0.5.3] - 2026-03-05

### Chore

- Switch release recipes to cargo install
- Simplify releasea recipe
- Remove Obsidian Base file
- Attach crumbs.skill to GitHub releases via publish recipe
- Bump version to 0.5.3

### Docs

- Add Obsidian Base for crumbs store
- Move Obsidian Base to repo root
- Add crumbs Claude skill for task management workflows

### Feat

- Case-insensitive ID lookup and glob prefix for global store

### Fix

- Use global_dir() comparison for glob prefix suggestion

## [0.5.2] - 2026-03-05

### Feat

- Allow multiple comma-separated targets in link command

## [0.5.1] - 2026-03-05

### Feat

- Add link command and --message flag to update

## [0.5.0] - 2026-03-05

### Feat

- Add export command with CSV, JSON, and TOON formats

## [0.4.1] - 2026-03-05

### Docs

- Update README for v0.4.0
- Add PowerShell completion instructions to README

### Feat

- Allow bare ID suffix without prefix in all commands

## [0.4.0] - 2026-03-05

### Chore

- Bump version to 0.4.0

### Feat

- Next command and due date support

## [0.3.0] - 2026-03-05

### CI

- Upload binaries to existing release on tag push

### Chore

- Bump version to 0.3.0

### Feat

- Edit, stats commands and --priority filter on list

## [0.2.0] - 2026-03-05

### Chore

- Bump dialoguer 0.11→0.12, toml 0.8→1.0
- Replace deprecated serde_yaml with serde_yml

### Feat

- V0.2.0 — color output for list and show

### Fix

- Rename --description to --message, short flag -m

## [0.1.9] - 2026-03-05

### Feat

- V0.1.9 — shell completions and -D for description

## [0.1.8] - 2026-03-05

### Feat

- V0.1.8 — dependency tracking via --depends

## [0.1.7] - 2026-03-05

### Feat

- V0.1.7 — delete, prefix, description, --all, --type

## [0.1.4] - 2026-03-05

### Feat

- Add --version / -V flag

## [0.1.3] - 2026-03-05

### Fix

- Respect --global flag for crumbs init

## [0.1.2] - 2026-03-05

### Chore

- Remove status.md

### Doc

- Use single quotes in examples, add shell tip

### Feat

- Friendly message when re-running init on existing store

### Fix

- Correct cargo strip flag in justfile

## [0.1.1] - 2026-03-05

### Chore

- Remove unused dependencies
- Bump version to 0.1.1

### Ci

- Add release workflow

### Doc

- Initial commit

<!-- generated by git-cliff -->
