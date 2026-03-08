---
id: cr-j90
title: 'GUI: autosave fires on blur even when body is unchanged'
status: closed
type: bug
priority: 1
tags:
- gui
- autosave
created: 2026-03-07
updated: 2026-03-07
closed_reason: 'Fixed: introduced loadedBody tracking in main.js; flushAutosave and scheduleAutosave now skip the save if detailText.value matches the last-loaded value'
dependencies: []
---

# GUI: autosave fires on blur even when body is unchanged

flushAutosave is called unconditionally on every blur event from the detail text area. If the GUI loaded an item with an empty description (e.g. before a CLI update had written the body), switching focus away from the window triggers a save of empty content — overwriting the real body with just the heading. Fix: track the last-loaded body text and skip the save if detailText.value matches it.
