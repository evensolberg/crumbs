---
id: cr-gia
title: 'GUI: add --danger CSS variable to theme blocks'
status: open
type: task
priority: 2
tags: []
created: 2026-04-10
updated: 2026-04-10
dependencies: []
phase: ''
---

# GUI: add --danger CSS variable to theme blocks

nav-chip-remove:hover uses var(--danger, #c0392b) but --danger is not defined in :root. The hardcoded fallback always fires. Define --danger (and a light-mode override) so themes can override it.
