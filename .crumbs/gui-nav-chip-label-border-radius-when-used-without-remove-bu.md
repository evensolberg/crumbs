---
id: cr-zzn
title: 'GUI: nav-chip-label border-radius when used without remove button'
status: closed
type: task
priority: 2
tags: []
created: 2026-04-10
updated: 2026-04-10
closed_reason: 'Fixed via CSS :last-child selector in PR #34'
dependencies: []
phase: ''
---

# GUI: nav-chip-label border-radius when used without remove button

When onRemove is absent the label button has border-radius: 10px 0 0 10px (left-only), which is fragile if a hover background is ever added. Should be full 10px when it is the only child. Low priority, no visual artifact today.
