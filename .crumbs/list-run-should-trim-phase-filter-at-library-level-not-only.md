---
id: cr-9ot
title: list::run should trim phase_filter at library level, not only in main.rs
status: closed
type: bug
priority: 3
tags:
- cli
- phase
created: 2026-04-05
updated: 2026-04-05
closed_reason: 'phase_filter trim moved to list::run; all-whitespace guard added; simplified to map+filter chain; merged in PR #17'
dependencies: []
phase: ''
---

# list::run should trim phase_filter at library level, not only in main.rs

[start] 2026-04-05 21:34:57  Trim phase_filter in list::run at library level

[stop]  2026-04-05 22:18:51  43m 54s  Fix complete: trim moved to library, all-whitespace guard, simplified to map+filter chain
