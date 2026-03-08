---
id: cr-ghd
title: 'CLI: bulk operations on filtered items'
status: open
type: feature
priority: 3
tags:
- cli
- bulk
created: 2026-03-07
updated: 2026-03-08
closed_reason: ''
dependencies: []
---

# CLI: bulk operations on filtered items

No way to update/close/tag multiple items matching a filter in one command. E.g. `crumbs update --status open --tag sprint/3 --set-priority 1` or `crumbs close --tag done`. Useful for triage and sprint management.

This should also apply to the GUI.

For the GUI, maybe also have a checkbox in front of each item and then apply the action. If nothing is checked, apply to the currently selected crumb.
