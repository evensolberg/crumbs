---
id: cr-74x
title: Reset tag and phase filters on store switch
status: in_progress
type: bug
priority: 3
tags:
- gui
created: 2026-04-05
updated: 2026-04-05
dependencies: []
phase: ''
---

# Reset tag and phase filters on store switch

switchStore() does not reset filterTag or filterPhase (nor their DOM inputs). Filters from store A silently carry over to store B. Fix: clear both filter variables and their input elements inside switchStore().

[2026-04-05] Investigate whether filters can be made persistent per directory. Maybe by updating the TOML file?

[start] 2026-04-05 21:39:54  Reset tag and phase filter inputs when switching stores in GUI
