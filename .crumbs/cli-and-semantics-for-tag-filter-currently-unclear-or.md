---
id: cr-vfb
title: 'CLI: AND semantics for --tag filter (currently unclear / OR)'
status: closed
type: bug
priority: 2
tags:
- cli
- filtering
created: 2026-04-04
updated: 2026-04-05
closed_reason: 'Fixed in PR #12'
dependencies: []
---

# CLI: AND semantics for --tag filter (currently unclear / OR)

crumbs list --tag security,backend is ambiguous — it is not documented or obvious whether this is AND (items that have both tags) or OR (items that have either). Practical use case: "show me all open security bugs" requires AND. Document the current behaviour and add --tag-mode and|or (or make comma = AND and pipe/space = OR). AND is the more useful default for triage.

[start] 2026-04-05 08:43:39

[stop]  2026-04-05 08:58:39  15m 0s  list.rs: split comma, .all() AND check; 2 new tests
