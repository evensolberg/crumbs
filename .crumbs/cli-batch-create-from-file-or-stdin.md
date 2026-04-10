---
id: cr-bxa
title: 'CLI: batch create from file or stdin'
status: closed
type: feature
priority: 2
tags:
- cli
- bulk
created: 2026-04-04
updated: 2026-04-09
closed_reason: 'Implemented in feat/batch-create-and-import (PR #30)'
dependencies: []
phase: ''
---

# CLI: batch create from file or stdin

No way to create multiple crumbs in one operation. When bootstrapping a backlog (e.g. from an audit or roadmap document), each crumb requires a separate command invocation. 

Add crumbs create --from <file.json|file.yaml> and/or crumbs create - (read JSON/YAML array from stdin).
Complements cr-ghd (bulk operations on filtered items) which covers the update side.

[2026-04-08] Implementation plan: (1) rename Command::Import→Pull in main.rs; (2) add create --from <file>|- subcommand with BatchCreateItem serde struct; (3) add import --file --format command with extension inference; (4) update skill, README, SKILL.md; (5) version bump. Skill/README/SKILL.md must be updated before closing.

[start] 2026-04-08 21:27:06

[stop]  2026-04-09 08:14:05  10h 46m 59s
