---
id: cr-w0h
title: Add milestone / phase field for roadmap grouping
status: in_progress
type: feature
priority: 2
tags:
- cli
- planning
created: 2026-04-04
updated: 2026-04-05
dependencies: []
---

# Add milestone / phase field for roadmap grouping

Crumbs are flat — there is no way to express "these items belong to Phase 1" or "Sprint 3" without using tags (lossy) or a separate document. Add a milestone field (string, e.g. "phase-1" or "2026-Q2") with crumbs list --milestone <name> and crumbs update --milestone <name>. Would make the roadmap document auto-generatable rather than hand-maintained.

[start] 2026-04-05 10:59:44  Adding milestone/phase field for roadmap grouping

[stop]  2026-04-05 11:14:24  14m 40s  Implemented phase field; PR opened
