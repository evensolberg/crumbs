---
id: cr-2m8
title: Add due/dependencies/blocks/blocked_by to reindex CSV
status: closed
type: feature
priority: 2
tags:
- cli
created: 2026-04-05
updated: 2026-04-06
closed_reason: 'blocks and blocked_by added to reindex CSV and export CSV; parity test added; merged in PR #22 (v0.19.2)'
dependencies: []
phase: ''
---

# Add due/dependencies/blocks/blocked_by to reindex CSV

index.csv reindex omits due, dependencies, blocks, blocked_by columns that export CSV includes. Since phase is now the stated motivation for external tooling, the index.csv schema gap limits that use case. Add the missing columns for parity.

[start] 2026-04-06 15:29:50  Adding due, dependencies, blocks, blocked_by columns to reindex CSV

[stop]  2026-04-06 16:21:13  51m 23s
