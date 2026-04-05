---
id: cr-3dl
title: 'GUI: Escape key propagation — global keydown fires alongside per-modal listeners'
status: closed
type: bug
priority: 3
tags:
- gui
- keyboard
created: 2026-03-15
updated: 2026-03-15
closed_reason: 'fixed: added e.stopPropagation() to all 12 per-modal/per-element Escape handlers in main.js'
dependencies: []
---

# GUI: Escape key propagation — global keydown fires alongside per-modal listeners

When any per-modal Escape listener fires (e.g. deleteModal keydown), the global document keydown handler also fires for the same event because neither calls e.stopPropagation(). Currently harmless — the global handler only hides the context menu and checks helpModal visibility. But if either handler gains more state-sensitive logic, double-execution could produce bugs. Fix options: (a) global handler checks isModalOpen() before acting on Escape, (b) per-modal listeners call e.stopPropagation().
