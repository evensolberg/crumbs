---
id: cr-qbd
title: 'CLI: Import from global to another directory doesn''t appear to work'
status: closed
type: bug
priority: 2
tags: []
created: 2026-03-28
updated: 2026-03-29
closed_reason: 'Fixed in PR #3 — import CLI dispatch had src/dst swapped'
dependencies: []
---

# CLI: Import from global to another directory doesn't appear to work

[start] 2026-03-29 17:15:40  Args to move_::run are swapped in Import handler — &dir and &src reversed

[stop]  2026-03-29 17:18:14  2m 34s  Fixed — src/dst swapped in main.rs Import dispatch
