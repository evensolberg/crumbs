# crumbs — Project Status

## Summary

Flat-folder Markdown task tracker written in Rust. A lightweight replacement for Beads with no daemon, no database — just `.md` files and a CSV cache.

## Current State

Initial implementation complete and compiling. Smoke-tested successfully.

## Implemented

- `crumbs create` — create a new item as a `.md` file with YAML frontmatter
- `crumbs list` — list items from the CSV cache, with optional status/tag filters
- `crumbs show <id>` — display a single item's details
- `crumbs update <id>` — update status, priority, or tags
- `crumbs close <id>` — mark an item closed with optional reason
- `crumbs reindex` — rebuild `index.csv` from all `.md` files in the directory
- `crumbs search <query>` — full-text search across `.md` file contents

## Design Decisions

- **Storage**: Flat folder, one `.md` file per item
- **File naming**: title-slug (collision → append ID suffix)
- **IDs**: Random 3-char suffix, prefixed `bc-` (e.g. `bc-x7q`)
- **CSV**: Cache only — rebuilt after every write; `crumbs reindex` to rebuild manually
- **Dependencies**: Stored in frontmatter as a list of IDs; informational only in v1
- **Tags**: Free-form; use `project/XYZ` convention for project grouping

## Frontmatter Schema

```yaml
---
id: bc-x7q
title: "Example item"
status: open        # open | in_progress | closed
type: task          # task | bug | feature | epic | idea
priority: 2         # 0=critical … 4=backlog
tags:
  - project/crumbs
created: 2026-03-05
updated: 2026-03-05
closed_reason: ""
dependencies: []
---
```

## Known Issues / Next Steps

- [ ] Fix unused import warning in `commands/update.rs`
- [ ] Add `--description` flag to `create` command (body content after frontmatter)
- [ ] Add `crumbs init` to create a directory and optionally a `.crumbs` marker file
- [ ] Shell completions
- [ ] `cargo install` / release binary
