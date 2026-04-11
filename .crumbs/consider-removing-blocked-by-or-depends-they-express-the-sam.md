---
id: cr-kwh
title: Consider removing blocked_by or depends - they express the same relationship
status: open
type: idea
priority: 3
tags:
- cli
- gui
- refactor
created: 2026-04-11
updated: 2026-04-11
dependencies: []
phase: ''
---

# Consider removing blocked_by or depends - they express the same relationship

[2026-04-11] Decision: remove 'depends' and keep 'blocked_by'/'blocks'. Rationale: blocked_by is bidirectional (both sides updated atomically via link_items), has status semantics (drives item to 'blocked' status), and is the stronger superset. 'depends' is unidirectional, purely cosmetic (no enforcement), and redundant. Removing it simplifies both the data model and the GUI bulk-edit panel.

Need to make sure to migrate any existing dependencies.
