---
id: cr-7fj
title: Crumbs import from JSON/TOON/CSV
status: closed
type: feature
priority: 3
tags: []
created: 2026-04-06
updated: 2026-04-09
closed_reason: 'Implemented in feat/batch-create-and-import (PR #30); TOON import tracked in cr-n4x'
dependencies: []
phase: ''
---

# Crumbs import from JSON/TOON/CSV

Need to be able to import crumbs from an external source via JSON/TOON/CSV.

[2026-04-08] Implementation plan: file-based import (inverse of export). Formats: json/toon/csv inferred from extension; --format required for unknown extensions or stdin. ID conflict handling: error on collision. Rename existing store-to-store import→pull. Skill/README/SKILL.md must be updated before closing.

[start] 2026-04-08 21:27:06

[stop]  2026-04-09 08:14:05  10h 46m 59s
