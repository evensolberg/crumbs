---
id: cr-7fj
title: Crumbs import from JSON/TOON/CSV
status: in_progress
type: feature
priority: 3
tags: []
created: 2026-04-06
updated: 2026-04-08
dependencies: []
phase: ''
---

# Crumbs import from JSON/TOON/CSV

Need to be able to import crumbs from an external source via JSON/TOON/CSV.

[2026-04-08] Implementation plan: file-based import (inverse of export). Formats: json/toon/csv inferred from extension; --format required for unknown extensions or stdin. ID conflict handling: error on collision. Rename existing store-to-store import→pull. Skill/README/SKILL.md must be updated before closing.

[start] 2026-04-08 21:27:06
