---
id: cr-ghd
title: 'CLI: bulk operations on filtered items'
status: closed
type: feature
priority: 2
tags:
- cli
- bulk
created: 2026-03-07
updated: 2026-04-10
closed_reason: 'Implemented in PR #32 — bulk update and close via filter flags with dry-run, confirmation prompt, and full Copilot review pass'
dependencies: []
phase: ''
---

# CLI: bulk operations on filtered items

No way to update/close/tag multiple items matching a filter in one command. E.g. `crumbs update --status open --tag sprint/3 --set-priority 1` or `crumbs close --tag done`. Useful for triage and sprint management.

This should also apply to the GUI. See cr-nk8
