# crumbs

A flat-folder Markdown task tracker written in Rust. A lightweight replacement for Beads with no daemon, no database — just `.md` files and a CSV cache.

## Concept

Each item is a plain `.md` file with YAML frontmatter. A `index.csv` acts as a read cache and is rebuilt after every write. There is no server process; everything is a file operation.

Items live either in a local `.crumbs/` directory (per-project) or a global store (`~/.local/share/crumbs`).

Because every item is a plain `.md` file, the store is trivially version-controlled. Commit `.crumbs/` to your repository and get full history, branching, and recovery for free via `git log`, `git diff`, and `git checkout`.

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

### Initialize a store

```sh
crumbs init                    # local store in .crumbs/
crumbs init --global           # global store at ~/.local/share/crumbs
crumbs init --prefix myp       # skip interactive prompt, set prefix directly
```

`crumbs init` prompts for an ID prefix (e.g. `cr`, `ma`), pre-filled from the directory name. Press Enter to accept or type your own. The prefix is saved to `.crumbs/config.toml` and used for all new item IDs in that store.

### Create an item

```sh
crumbs create 'Fix the login bug' --item-type bug --priority 1 --tags project/auth
crumbs create 'Auth redesign' --message 'Covers login, OAuth, and session handling'
crumbs create 'Ship it' --due 2026-04-01
crumbs c 'Quick idea'          # shorthand
# Tip: use single quotes to avoid shell expansion of special characters (!, $, etc.)
```

| Flag | Default | Values |
|------|---------|--------|
| `-t, --item-type` | `task` | `task`, `bug`, `feature`, `epic`, `idea` |
| `-p, --priority` | `2` | `0` (critical) … `4` (backlog) |
| `--tags` | — | comma-separated, e.g. `project/foo,needs-review` |
| `-m, --message` | — | freeform text stored in the markdown body |
| `--depends` | — | comma-separated dependency IDs, e.g. `cr-abc,cr-xyz` |
| `--due` | — | due date in `YYYY-MM-DD` format |

### List items

```sh
crumbs list                    # open and in-progress only
crumbs list --all              # include closed
crumbs list --status open
crumbs list --tag project/auth
crumbs list --priority 0       # show only P0 items
```

Overdue items are flagged with `!due` in bold red. Items with a future due date show `due:YYYY-MM-DD`.

### Show an item

```sh
crumbs show bc-x7q
```

### Show next item

```sh
crumbs next
```

Shows the highest-priority open item (sorted by priority, then age). Useful for a quick "what should I work on?" answer.

### Update an item

```sh
crumbs update bc-x7q --status in_progress
crumbs update bc-x7q --priority 0
crumbs update bc-x7q --tags project/auth,urgent
crumbs update bc-x7q --type bug
crumbs update bc-x7q --depends cr-abc,cr-xyz
crumbs update bc-x7q --due 2026-04-01
crumbs update bc-x7q --clear-due
```

### Edit an item

```sh
crumbs edit bc-x7q
```

Opens the item's `.md` file in `$EDITOR` (falls back to `$VISUAL`, then `vi`). The index is rebuilt automatically after the editor exits.

### Close an item

```sh
crumbs close bc-x7q
crumbs close bc-x7q --reason "fixed in PR #42"
```

### Delete an item

```sh
crumbs delete cr-x7q              # delete a specific item
crumbs delete --closed            # delete all closed items at once
```

### Statistics

```sh
crumbs stats
```

Prints a summary of items by status, type, and priority with color coding.

### Search

```sh
crumbs search "login"
```

Full-text search across all `.md` file contents and titles.

### Export

```sh
crumbs export                        # JSON to stdout (default)
crumbs export --format csv           # CSV to stdout
crumbs export --format toon          # TOON to stdout
crumbs export --format json --output items.json
```

Exports all items. TOON (Token-Oriented Object Notation) is a compact, token-efficient format suited for feeding into LLMs.

### Shell completions

Generate and install a completion script once, then enjoy tab-completion for all subcommands and flags.

#### zsh

```sh
crumbs completions zsh > ~/.zfunc/_crumbs
# Add to ~/.zshrc if not already present:
#   fpath=(~/.zfunc $fpath)
#   autoload -U compinit && compinit
```

#### bash

```sh
crumbs completions bash > ~/.local/share/bash-completion/completions/crumbs
```

#### fish

```sh
crumbs completions fish > ~/.config/fish/completions/crumbs.fish
```

#### PowerShell

```powershell
crumbs completions powershell >> $PROFILE
```

Or save to a file and dot-source it from your profile:

```powershell
crumbs completions powershell > ~/crumbs_completion.ps1
# Add to $PROFILE:
#   . ~/crumbs_completion.ps1
```

After reinstalling crumbs, regenerate the file with the same command.

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
due: 2026-04-01     # optional
---
```

## File naming

Files are named after a slug of the title (max 60 chars). On collision the item ID suffix is appended, e.g. `my-task-x7q.md`.

## Global flags

| Flag | Description |
|------|-------------|
| `-d, --dir <path>` | Use a specific directory |
| `-g, --global` | Use the global store |
