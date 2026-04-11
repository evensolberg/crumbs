---
id: cr-2zp
title: 'GUI: Dragging a crumb from one store to another doesn’t change the prefix'
status: in_progress
type: bug
priority: 2
tags: []
created: 2026-04-10
updated: 2026-04-11
dependencies: []
phase: ''
---

# GUI: Dragging a crumb from one store to another doesn’t change the prefix

[start] 2026-04-11 08:05:09  Root cause: clearDropTargets() called before sidebarTargetAt() in onUp. If cursor is 1px off sidebar on mouseup, elementFromPoint misses, target=null, move silently skipped. Fix: capture target (with .drop-target fallback) before clearing.
