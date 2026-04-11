---
id: cr-kwh
title: Consider removing blocked_by or depends - they express the same relationship
status: closed
type: idea
priority: 3
tags:
- cli
- gui
- refactor
created: 2026-04-11
updated: 2026-04-11
closed_reason: 'Removed depends field; auto-migrates to blocked_by/blocks bidirectionally on first load_all. PR #37.'
phase: ''
---

# Consider removing blocked_by or depends - they express the same relationship

[2026-04-11] Decision: remove 'depends' and keep 'blocked_by'/'blocks'. Rationale: blocked_by is bidirectional (both sides updated atomically via link_items), has status semantics (drives item to 'blocked' status), and is the stronger superset. 'depends' is unidirectional, purely cosmetic (no enforcement), and redundant. Removing it simplifies both the data model and the GUI bulk-edit panel.

Need to make sure to migrate any existing dependencies.

[start] 2026-04-11 13:19:58  Starting removal of depends field; keeping blocked_by/blocks. Full bidirectional promotion on read.

[2026-04-11] Implementation plan: docs/superpowers/plans/2026-04-11-remove-depends.md — 5 tasks. Key decision: two-pass migration in load_all so all items are in memory before any migrate_depends call, avoiding filesystem readdir order dependency.

[stop]  2026-04-11 14:12:32  52m 34s
