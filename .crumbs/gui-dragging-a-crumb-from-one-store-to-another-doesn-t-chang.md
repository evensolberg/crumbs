---
id: cr-2zp
title: 'GUI: Dragging a crumb from one store to another doesn’t change the prefix'
status: closed
type: bug
priority: 2
tags: []
created: 2026-04-10
updated: 2026-04-11
closed_reason: 'Fixed: resolve drop target before clearDropTargets() in onUp; fallback to .store-item[data-path].drop-target with storeDir guard'
dependencies: []
phase: ''
---

# GUI: Dragging a crumb from one store to another doesn’t change the prefix

[start] 2026-04-11 08:05:09  Root cause: clearDropTargets() called before sidebarTargetAt() in onUp. If cursor is 1px off sidebar on mouseup, elementFromPoint misses, target=null, move silently skipped. Fix: capture target (with .drop-target fallback) before clearing.

[stop]  2026-04-11 09:13:00  1h 7m 51s
