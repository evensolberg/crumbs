---
name: crumbs
description: Use the crumbs CLI to manage tasks, bugs, features, and ideas tracked as plain Markdown files in a .crumbs/ store. Use this skill when the user asks to create, list, show, update, close, delete, link, search, export, or time-track crumbs items — or asks what to work on next. Also use when asked to initialize a crumbs store in a project. crumbs is already installed; run it directly, no cargo run needed.
---

# crumbs

Flat-folder Markdown task tracker. Each item is a `.md` file with YAML frontmatter. A `index.csv` is the read cache, rebuilt after every write. No daemon, no database.

Store lives in `.crumbs/` (local, auto-detected from cwd) or the platform global store:
- macOS: `~/Library/Application Support/crumbs`
- Linux: `~/.local/share/crumbs`
- Windows: `%APPDATA%\crumbs`

## Store resolution

1. `--dir <path>` explicit override
2. `--global` → platform global store (see above)
3. `.crumbs/` under cwd (auto-detected)
4. Global store as fallback

## Common workflows

### Initialize
```sh
crumbs init                      # local store, prompts for prefix
crumbs init --prefix myp         # skip prompt
crumbs init --global             # global store (prefix suggestion: "glob")
```

### Create
```sh
crumbs create 'Fix login bug' --type bug --priority 1 --tags auth
crumbs create 'Auth redesign' --message 'Covers OAuth and sessions' --due 2026-04-01
crumbs c 'Quick idea'            # shorthand
```
Title is a positional argument on `create` — there is no `--title` flag. (`--title` is only available on `update`.)
Use single quotes to avoid shell expansion of `!`, `$`, etc.

### List & triage
```sh
crumbs list                      # open + in_progress + blocked + deferred
crumbs list --all
crumbs list --status blocked
crumbs list --status open --priority 0
crumbs list --tag project/auth
crumbs list --type bug           # filter by type (task, bug, feature, epic, idea)
crumbs list --phase phase-1      # filter by phase label
crumbs list --verbose            # show first two body lines beneath each item
crumbs list --sort priority      # sort by: id (default), priority, status, title, type, due, created, updated
crumbs next                      # highest-priority actionable item (skips deferred with future until date)
```

### Inspect
```sh
crumbs show cr-x7q               # IDs are case-insensitive
crumbs show cr-x7q cr-y8r cr-z9s # show multiple items
crumbs stats
crumbs search "login"
```

### Update
```sh
crumbs update cr-x7q --title 'New title'
crumbs update cr-x7q --status in_progress
crumbs update cr-x7q --priority 0 --tags auth,urgent
crumbs update cr-x7q --message 'Now includes OAuth flow'
crumbs update cr-x7q --append 'See PR #99'             # appends with [date] prefix
crumbs update cr-x7q --due 2026-04-01
crumbs update cr-x7q --clear-due
crumbs update cr-x7q --depends cr-abc,cr-xyz
crumbs update cr-x7q --type bug
crumbs update cr-x7q --points 5
crumbs update cr-x7q --clear-points
crumbs update cr-x7q --phase phase-1
crumbs update cr-x7q --phase 2026-Q2
crumbs update cr-x7q --clear-phase
```

### Append (shorthand)
```sh
crumbs append cr-x7q 'Quick note'   # shorthand for update --append; alias: a
crumbs a cr-x7q 'Quick note'
```

### Block and defer
```sh
crumbs block cr-x7q cr-y8r,cr-z9s   # cr-x7q blocks targets; targets get blocked status
crumbs block cr-x7q cr-y8r --remove  # unlink; targets reopen if nothing else blocks them
crumbs block cr-x7q                   # mark cr-x7q itself as blocked (no link)
crumbs defer cr-x7q                          # set status to deferred
crumbs defer cr-x7q --until 2026-04-01       # defer with a wake-up date; resurfaces in next
crumbs defer cr-x7q --reopen                 # reopen a deferred item
```

### Move and import between stores
```sh
crumbs move cr-x7q --to /path/to/other/.crumbs   # move to another store (new ID)
crumbs move cr-x7q --to global                    # move to the global store
crumbs import glob-x7q --from global              # import from global into current store
crumbs import glob-x7q --from /path/to/.crumbs   # import from a specific store
```

### Link (blocking relationships)
```sh
crumbs link cr-x7q blocks cr-y8r            # bidirectional; sets cr-y8r to blocked
crumbs link cr-x7q blocks cr-y8r,cr-z9s    # multiple targets
crumbs link cr-x7q blocked-by cr-z9s
crumbs link cr-x7q blocks cr-y8r --remove  # unlink; restores open if unblocked
```

### Time tracking

```sh
crumbs start cr-x7q                                        # append [start] entry, set in_progress
crumbs start cr-x7q -m 'Investigating root cause'
crumbs stop  cr-x7q                                        # append [stop] with elapsed time
crumbs stop  cr-x7q -m 'Fixed, needs review'
crumbs show  cr-x7q                                        # shows "Total tracked: Xh Ym Zs"
```

Start/stop entries are plain lines in the markdown body, interleaved with any notes added via `--append`. A typical item body looks like:

```
[2026-03-08] Reproduced locally.
[start] 2026-03-08 09:00:00  Investigating root cause
[2026-03-08] Found the bug in main.js line 401.
[stop]  2026-03-08 09:47:12  47m 12s  Fixed, needs review
```

`crumbs start` errors with "Already started at HH:MM:SS" if an unmatched `[start]` exists.

### Close / delete / clean
```sh
crumbs close cr-x7q --reason "fixed in PR #42"
crumbs delete cr-x7q
crumbs clean                     # purge all closed items
```

### Export
```sh
crumbs export                          # JSON to stdout
crumbs export --format csv
crumbs export --format toon            # token-efficient for LLMs
crumbs export --format json --output items.json
crumbs export --output                 # → crumbs_export.json (default filename)
crumbs export --format csv --output    # → crumbs_export.csv
```

### Edit title and body (TUI)
```sh
crumbs body cr-x7q               # inline TUI editor: line 1 = title, rest = body; Ctrl-S saves, Esc exits
```

### Edit raw file
```sh
crumbs edit cr-x7q               # opens full .md file in $EDITOR; reindexes on exit
crumbs reindex                   # rebuild index.csv manually
```

## Item schema

| Field | Values |
|-------|--------|
| `status` | `open`, `in_progress`, `blocked` (⊘), `deferred` (◷), `closed` |
| `type` | `task`, `bug`, `feature`, `epic`, `idea` |
| `priority` | `0`=critical, `1`=high, `2`=normal, `3`=low, `4`=backlog |
| `tags` | comma-separated, e.g. `project/auth,urgent` |
| `due` | `YYYY-MM-DD` |
| `dependencies` | comma-separated IDs |
| `blocks` / `blocked_by` | set via `link` or `block` command |
| `story_points` | optional integer; conventional Fibonacci values: 1, 2, 3, 5, 8, 13, 21 |
| `phase` | optional string; free-form label, e.g. `phase-1`, `2026-Q2` |

## Time tracking format

Timer entries live in the markdown body alongside other notes:

| Line | Format |
|------|--------|
| Start | `[start] YYYY-MM-DD HH:MM:SS  [optional comment]` |
| Stop  | `[stop]  YYYY-MM-DD HH:MM:SS  Xh Ym Zs  [optional comment]` |

`crumbs show` sums all matched pairs and prints `Total tracked: Xh Ym Zs`.

## Key behaviors

- IDs are **case-insensitive**: `CR-X7Q` == `cr-x7q`
- Bare suffix works: `crumbs show x7q` expands to `{prefix}-x7q`
- `crumbs show` accepts multiple IDs: `crumbs show x7q y8r z9s`
- `link blocks` and `block` update **both** items atomically and set blocked status on targets
- Unlinking restores `open` on targets when no other blockers remain
- `--tags` and `--depends` on update **replace** the existing list (not append)
- `--tag` on list uses **AND semantics**: `--tag alpha,beta` returns only items that have both tags; empty parts are ignored
- `--append 'text'` adds to the body with a `[YYYY-MM-DD]` timestamp prefix; `--message 'text'` replaces it
- `:shortcode:` in body text (message, append, timer comments) is expanded to Unicode at write time — e.g. `:tada:` → 🎉, `:bug:` → 🐛, `:+1:` → 👍; unknown shortcodes pass through unchanged
- `crumbs defer --until <date>` sets the due date; `crumbs next` skips deferred items with a future until date and skips items whose `blocked_by` items are still open
- `crumbs start` / `crumbs stop` append timer entries to the body; `crumbs show` sums elapsed time as "Total tracked"
- File names are title slugs; collisions get the ID suffix appended
- `.crumbs/` can be committed to git for full history
- `move`/`import` reassign a new ID using the destination store's prefix
- Use `"global"` as the path for `--to` / `--from` to refer to the global store
