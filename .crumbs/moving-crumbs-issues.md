---
id: cr-7w2
title: Moving crumbs issues
status: in_progress
type: task
priority: 2
tags: []
created: 2026-03-09
updated: 2026-03-09
closed_reason: ''
dependencies: []
---

# Moving crumbs issues

1. Drag and drop in the GUI doesn’t appear to work. I tried to drag something from the list in one directory onto the target in the STORES list and it didn’t want to drop it there.
2. Moving crumbs from one directory to another in the CLI does work but:
  1. We need to change the order of the parameters. Currently it’s `crumbs move —to <dir> <ID>`. This is more natural: `crumbs move <ID> —to <dir>`
  2. The cumb needs to change the ID when it moves. Currently it retains the old one so if I move it from prefix `abc` to prefix `xyz`, it retains the old prefix and isn’t seen at the new location.

[start] 2026-03-09 17:40:37  Fix GUI drag-and-drop: add dragenter handler + fix dragleave to check relatedTarget

[2026-03-09] Fixed:
1. CLI move/import: --to/--from now routed through resolve_dir so project dirs auto-resolve to .crumbs/. Was using PathBuf::from(&to) directly.
2. GUI drag-and-drop: added dragenter (calls e.preventDefault() + adds drop-target class); fixed dragleave to check e.relatedTarget so class is only removed when cursor truly leaves, not when it moves onto a child element (store-name or store-item-path spans).
3. CLI arg order (cr said "move --to <dir> <id>") already fixed in current code.
4. Cleaned up stray index.csv from causal_loop project root (created by buggy reindex call on project root).

[stop]  2026-03-09 17:45:55  5m 18s  All three sub-issues addressed
