---
id: cr-46u
title: pre-commit hook stages every modified file, not just fmt output
status: open
type: bug
priority: 2
tags:
- tooling
- git
created: 2026-11-13
updated: 2026-11-13
phase: ''
---

# pre-commit hook stages every modified file, not just fmt output

The pre-commit hook runs `cargo fmt` and then `git add` on every path in `git diff --name-only`. Any unstaged edit to a tracked file (e.g. pending .crumbs/ changes) is swept into whatever commit is being made. Seen 2026-11-13 when crumb closures landed in the cr-oar Fix: commit.

Fix: capture `git diff --name-only` before running fmt, and re-stage only staged files that fmt changed (e.g. intersect `git diff --cached --name-only` with files modified by fmt).
