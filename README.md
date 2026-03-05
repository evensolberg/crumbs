# crumbs

A flat-folder Markdown task tracker written in Rust. A lightweight replacement for Beads with no daemon, no database — just `.md` files and a CSV cache.

## Concept

Each item is a plain `.md` file with YAML frontmatter. A `index.csv` acts as a read cache and is rebuilt after every write. There is no server process; everything is a file operation.

Items live either in a local `.crumbs/` directory (per-project) or a global store (`~/.local/share/crumbs`).

## Installation

```sh
cargo install --path .
```

## Store resolution

crumbs locates its store in this order:

1. `--dir <path>` — explicit override
2. `--global` — global store at `~/.local/share/crumbs`
3. `.crumbs/` under the current directory (auto-detected)
4. Global store as fallback

## Usage

### Initialize a local store

```sh
crumbs init
```

Creates a `.crumbs/` directory in the current directory.

### Create an item

```sh
crumbs create "Fix the login bug" --item-type bug --priority 1 --tags project/auth
crumbs c "Quick idea"          # shorthand
```

| Flag | Default | Values |
|------|---------|--------|
| `-t, --item-type` | `task` | `task`, `bug`, `feature`, `epic`, `idea` |
| `-p, --priority` | `2` | `0` (critical) … `4` (backlog) |
| `--tags` | — | comma-separated, e.g. `project/foo,needs-review` |

### List items

```sh
crumbs list
crumbs list --status open
crumbs list --tag project/auth
```

### Show an item

```sh
crumbs show bc-x7q
```

### Update an item

```sh
crumbs update bc-x7q --status in_progress
crumbs update bc-x7q --priority 0
crumbs update bc-x7q --tags project/auth,urgent
```

### Close an item

```sh
crumbs close bc-x7q
crumbs close bc-x7q --reason "fixed in PR #42"
```

### Search

```sh
crumbs search "login"
```

Full-text search across all `.md` file contents and titles.

### Reindex

```sh
crumbs reindex
```

Rebuilds `index.csv` from all `.md` files. Useful if files were edited manually or the cache is stale.

## Frontmatter schema

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

## File naming

Files are named after a slug of the title (max 60 chars). On collision the item ID suffix is appended, e.g. `my-task-x7q.md`.

## Global flags

| Flag | Description |
|------|-------------|
| `-d, --dir <path>` | Use a specific directory |
| `-g, --global` | Use the global store |
