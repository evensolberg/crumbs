---
id: cr-tbd
title: 'crumbs next: skip items whose blockers are still open'
status: closed
type: bug
priority: 2
tags:
- cli
- dependencies
created: 2026-04-04
updated: 2026-04-05
closed_reason: 'Fixed in PR #12'
dependencies: []
---

# crumbs next: skip items whose blockers are still open

crumbs next picks the highest-priority actionable item but does not check whether its dependencies/blockers are resolved. If vsm-abc blocks vsm-def and vsm-abc is still open, crumbs next can surface vsm-def even though it cannot be started. next should skip any item that has an unresolved blocks_by dependency, consistent with the existing deferred --until logic.

[start] 2026-04-05 08:43:39

[stop]  2026-04-05 08:58:39  15m 0s  next.rs: HashMap status lookup + blocked_by filter; 2 new tests
