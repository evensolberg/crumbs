---
id: cr-46u
title: pre-commit hook stages every modified file, not just fmt output
status: closed
type: bug
priority: 2
tags:
- tooling
- git
created: 2026-11-13
updated: 2026-11-13
closed_reason: 'Fixed: local hook formats staged .rs content via rustfmt stdin and re-stages only fully staged files; partly staged files needing formatting abort. Saved with tests to ~/.claude/scripts/git-hooks/rust-fmt-staged.'
phase: ''
---

# pre-commit hook stages every modified file, not just fmt output

The pre-commit hook runs `cargo fmt` and then `git add` on every path in `git diff --name-only`. Any unstaged edit to a tracked file (e.g. pending .crumbs/ changes) is swept into whatever commit is being made. Seen 2026-11-13 when crumb closures landed in the cr-oar Fix: commit.

Fix: capture `git diff --name-only` before running fmt, and re-stage only staged files that fmt changed (e.g. intersect `git diff --cached --name-only` with files modified by fmt).

[start] 2026-11-13 22:09:53  Hook rewritten to format and re-stage only staged .rs files

[stop]  2026-11-13 22:16:54  7m 1s
