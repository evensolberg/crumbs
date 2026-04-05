---
id: cr-rgl
title: 'Refactor: split main() to reduce line count'
status: closed
type: task
priority: 3
tags:
- cli
- lint
created: 2026-03-29
updated: 2026-03-31
closed_reason: 'Resolved in PR #9'
dependencies: []
---

# Refactor: split main() to reduce line count

main() in crumbs/src/main.rs is 220+ lines (limit 100). Extract command dispatch arms into a dedicated dispatch() function or per-command handler modules to bring it under the clippy::too_many_lines threshold.

[start] 2026-03-31 07:57:26  Splitting alongside cr-mop; one PR for both too_many_lines fixes

[stop]  2026-03-31 20:45:36  12h 48m 10s  Done via PR #9
