---
id: cr-el7
title: 'GUI: native menu bar for all operations'
status: open
type: feature
priority: 3
tags:
- gui
- ux
created: 2026-04-08
updated: 2026-04-08
dependencies: []
phase: ''
---

# GUI: native menu bar for all operations

For OS compliance, the GUI should expose all toolbar operations (New, Start, Block, Defer, Timer, Close, Delete, Next, Clean closed, Import, Export, Reindex) as native menu items in addition to toolbar buttons. Tauri v2 supports a native menu via tauri::menu::Menu. Operations that require a selected item should be greyed out when nothing is selected (same logic as the disabled button states). Keyboard shortcuts already defined in the app should be surfaced in the menu so users can discover them.
