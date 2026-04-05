# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

## [0.17.3] - 2026-04-05

### Chore

- Bump version to 0.17.3

### Fix

- `crumbs next` now skips items whose blockers are still open (cr-tbd)
- `crumbs list --tag` now uses AND semantics for comma-separated values (cr-vfb)
- GUI: markdown preview now renders top-aligned instead of vertically centred (cr-99g)

## [0.17.2] - 2026-03-31

### Chore

- Bump flatted from 3.4.1 to 3.4.2 in /crumbs-gui
- Bump version to 0.17.2

### Docs

- Sync README and skill with current CLI
- Tighten --sort help; annotate CreateArgs default
- Document run_structured_commands invariant (cr-mvk)

### Feat

- Add ValueEnum to SortKey for shell completions

### Fix

- Swap Type and Priority order in Properties panel
- Swap Type and Priority filter order in toolbar
- Mirror Type/Priority swap in dist files; restore CHANGELOG
- Make SortKey Display infallible (Copilot review on PR #10)
- Use vec! for ValueEnum test comparison (Copilot review on PR #10)
- Move closed_reason to body when reopening a crumb
- Skip serializing empty closed_reason; add append+reopen test

### Refactor

- Bundle create::run args into CreateArgs struct (#8)
- Split expand_shortcodes and main() to fix too_many_lines (#9)
- Tighten SortKey Display and test per review
- Extract apply_status helper; add edge-case tests

## [0.17.1] - 2026-03-30

### Chore

- Merge chrono imports and add append subcommand test
- Bump version to 0.17.1

### Docs

- Document append subcommand in README and skill

### Feat

- Add append subcommand and bold ID in create output
- GUI polish, cursor fix, date button, resizable outline panel (#5)

### Fix

- Append subcommand alias and confirmation message
- Drop .white() from create ID style for theme consistency
- Address Copilot review comments on append PR
- Remove output_label from test that doesn't assert on it
- Move output_label out of UpdateArgs into a run() parameter
- Restore run() signature; add run_labeled() for append subcommand
- Hide run_labeled from API docs; collapse version bumps in changelog

## [0.16.5] - 2026-03-16

### Chore

- Export snippet from CM6 bundle
- Bump version to 0.16.3
- Bump version to 0.16.4
- Bump version to 0.16.5

### Docs

- Add keyboard shortcuts & help modal design spec

### Feat

- Toolbar restructure and help modal HTML
- Help modal styles
- Keyboard shortcuts and help modal

### Fix

- Code review — modal guards, openDeleteModal, kbd scope, blank line
- Cmd+F modal guard, clarify Cmd+F help text
- Address Copilot review comments on keyboard shortcuts PR
- Address second round of Copilot review comments
- Address third round of Copilot review comments
- Address fourth round of Copilot review comments
- Improve help modal layout — wider, less cramped
- Improve kbd styling consistency in help modal
- Stop Escape propagation in all per-modal/per-element handlers

## [0.16.2] - 2026-03-12

### Feat

- V0.16.2 — list --type, update --title, fix move path resolution

## [0.16.1] - 2026-03-10

### Chore

- Bump version to 0.16.1

### Feat

- Redesign body editor UX
- Replace double-Esc discard with explicit S/E/R prompt

### Fix

- Use "resume" instead of "cancel" in discard prompt
- Use Alt+b/f for word navigation (macOS terminal compatibility)

## [0.16.0] - 2026-03-10

### Chore

- Add ratatui, ratatui-textarea, and crossterm deps
- Bump version to 0.16.0; fix lint warnings in body.rs
- Add ESLint to crumbs-gui; wire into just lint; fix JS warnings

### Docs

- Document crumbs body command in README

### Feat

- Add body command helper functions with unit tests
- Implement body command TUI editor
- Wire crumbs body command into CLI

## [0.15.1] - 2026-03-10

### Chore

- Add CodeMirror 6 bundle for GUI editor upgrade
- Bump version to 0.15.0
- Move emoji button into the format toolbar
- Bump version to 0.15.1

### Docs

- Add reference to Pragmatic Rust Guidelines in CLAUDE.md
- Add design spec for GUI editor enhancements (cr-4eq, cr-8r8)
- Add implementation plan for GUI editor enhancements

### Feat

- Replace textarea with CodeMirror 6 editor in GUI
- Color-code item types in GUI table (cr-kos)
- Enable line wrapping in GUI editor
- Add format toolbar and editor keyboard shortcuts to GUI
- Add editor keyboard shortcuts (delete/move line)
- Add Mod-0 shortcut for normal (non-heading) text
- Color-code headings by level, remove underline in editor

### Fix

- Walk ancestor dirs to find local .crumbs store (cr-v7e)

### Test

- Add bare suffix lookup tests for find_by_id and show

## [0.14.5] - 2026-03-10

### Chore

- Bump version to 0.14.5

### Fix

- GUI drag-and-drop via event delegation + module dragItemId
- Replace HTML5 DnD with pointer-event drag simulation for GUI

## [0.14.4] - 2026-03-10

### Chore

- Add emoji module, new crumb items, update dist and changelog

### Fix

- Move/import resolve destination via resolve_dir (v0.14.4)

## [0.14.3] - 2026-03-09

### Feat

- Default export filename crumbs_export.<ext> for --output
- GUI Created/Updated table columns (v0.14.3)

### Fix

- Add dialog:allow-save permission; left-align status/priority cols
- Change default export filename to crumbs_export.<ext>

### Refactor

- Extract rewrite_frontmatter, collapse Create/C, split_csv

## [0.13.2] - 2026-03-08

### Chore

- Finalize CHANGELOG for v0.12.0

### Docs

- Update README, SKILL.md, CLAUDE.md for time tracking (v0.12.0)
- Show ID before -m comment in start/stop examples

### Feat

- Start/stop timer + fix description in frontmatter (v0.11.6)

### Fix

- Make start/stop comment a positional arg (no -m flag needed)
- Restore -m flag for start/stop comment so ID can go last

## [0.11.5] - 2026-03-08

### Feat

- Multi-show, list --verbose, --append, defer --until (v0.11.5)

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
