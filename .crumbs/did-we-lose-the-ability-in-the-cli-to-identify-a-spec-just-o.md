---
id: cr-v7e
title: Did we lose the ability in the CLI to identify a spec just on the last 3 letters when in a directory with a .crumbs initialized?
status: closed
type: bug
priority: 2
tags: []
created: 2026-03-09
updated: 2026-03-10
closed_reason: 'fixed: resolve_dir now walks ancestor dirs like git does for .git'
dependencies: []
---

# Did we lose the ability in the CLI to identify a spec just on the last 3 letters when in a directory with a .crumbs initialized?

[2026-03-10] Investigation (2026-03-10): find_by_id in store.rs correctly implements bare-suffix expansion. All new tests pass (98/98). Root cause: installed binary likely predates the bare-suffix feature or the worktree cwd is not the project root. Fix: run just release to rebuild.
