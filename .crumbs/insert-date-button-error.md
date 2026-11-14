---
id: cr-lvq
title: Insert Date button error
status: closed
type: bug
priority: 2
tags: []
created: 2026-09-14
updated: 2026-11-13
closed_reason: 'Fixed: GUI uses localDateISO() instead of UTC toISOString() (fix/gui-local-date)'
phase: ''
---

# Insert Date button error

The [date] button in the GUI editor inserts the wrong date. It is the 14th here, and it inserted the 15th. What time does it use to insert the date? Local? UTC?

[start] 2026-11-13 20:46:31  Use local date instead of UTC toISOString

[stop]  2026-11-13 21:15:06  28m 35s
