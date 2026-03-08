---
name: crumbs
description: Use the crumbs CLI to manage tasks, bugs, features, and ideas tracked as plain Markdown files in a .crumbs/ store. Use this skill when the user asks to create, list, show, update, close, delete, link, search, or export crumbs items — or asks what to work on next. Also use when asked to initialize a crumbs store in a project. crumbs is already installed; run it directly, no cargo run needed.
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
crumbs create 'Fix login bug' --item-type bug --priority 1 --tags auth
crumbs create 'Auth redesign' --message 'Covers OAuth and sessions' --due 2026-04-01
crumbs c 'Quick idea'            # shorthand
```
Use single quotes to avoid shell expansion of `!`, `$`, etc.

### List & triage
```sh
crumbs list                      # open + in_progress + blocked + deferred
crumbs list --all
crumbs list --status blocked
crumbs list --status open --priority 0
crumbs list --tag project/auth
crumbs list --verbose            # show first two body lines beneath each item
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

### Close / delete
```sh
crumbs close cr-x7q --reason "fixed in PR #42"
crumbs delete cr-x7q
crumbs delete --closed           # purge all closed items
```

### Export
```sh
crumbs export                          # JSON to stdout
crumbs export --format csv
crumbs export --format toon            # token-efficient for LLMs
crumbs export --format json --output items.json
```

### Edit raw file
```sh
crumbs edit cr-x7q               # opens in $EDITOR; reindexes on exit
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

## Key behaviors

- IDs are **case-insensitive**: `CR-X7Q` == `cr-x7q`
- Bare suffix works: `crumbs show x7q` expands to `{prefix}-x7q`
- `crumbs show` accepts multiple IDs: `crumbs show x7q y8r z9s`
- `link blocks` and `block` update **both** items atomically and set blocked status on targets
- Unlinking restores `open` on targets when no other blockers remain
- `--tags` and `--depends` on update **replace** the existing list (not append)
- `--append 'text'` adds to the body with a `[YYYY-MM-DD]` timestamp prefix; `--message 'text'` replaces it
- `crumbs defer --until <date>` sets the due date; `crumbs next` skips deferred items with a future until date
- File names are title slugs; collisions get the ID suffix appended
- `.crumbs/` can be committed to git for full history
- `move`/`import` reassign a new ID using the destination store's prefix
- Use `"global"` as the path for `--to` / `--from` to refer to the global store
