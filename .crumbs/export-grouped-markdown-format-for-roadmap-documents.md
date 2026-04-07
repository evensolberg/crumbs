---
id: cr-rxy
title: 'Export: grouped markdown format for roadmap documents'
status: closed
type: feature
priority: 2
tags:
- cli
- export
- planning
created: 2026-04-04
updated: 2026-04-06
dependencies: []
phase: ''
resolution: evensolberg/crumbs#25
---

# Export: grouped markdown format for roadmap documents

crumbs export --format toon is good for LLM context but there is no format that produces a human-readable grouped document. Add crumbs export --format markdown --group-by type,priority (or --group-by milestone,priority) that outputs a markdown table or section-per-group layout. Would make the production-grade-roadmap.md style document auto-generatable from the crumbs store rather than maintained by hand.

[2026-04-04] See /Volumes/SSD/Source/vsm-studio/docs/plans/production-grade-roadmap.md for an example.

[start] 2026-04-06 21:15:08  Starting implementation

[stop]  2026-04-06 21:18:58  3m 50s  Implemented; PR pending

[start] 2026-04-06 21:46:22

[stop]  2026-04-06 22:04:49  18m 27s  All review threads resolved
