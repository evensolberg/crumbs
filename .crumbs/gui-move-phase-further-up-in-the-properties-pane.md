---
id: cr-kel
title: 'GUI: Move Phase further up in the Properties pane'
status: in_progress
type: task
priority: 2
tags: []
created: 2026-04-05
updated: 2026-04-05
closed_reason: 'Phase field now appears after status across CLI show, CSV export, reindex CSV, GUI columns, detail pane, and toolbar filters; merged in PR #14'
dependencies: []
phase: ''
---

# GUI: Move Phase further up in the Properties pane

Should sit under Status. That way we get Phase, Type (Task/Feature, ...), Priority

The same order should be in the table. This also means that Phase should come before the Type and Priority filters in the toolbar.

[start] 2026-04-05 14:10:06  Reordering phase field across all surfaces

[stop]  2026-04-05 14:24:27  14m 21s  Phase field reordered across all surfaces; PR #14 merged

[start] 2026-04-05 14:32:06  Move phase from trailing @marker to inline badge [P1][phase][type] in list output

[stop]  2026-04-05 14:32:22  16s
