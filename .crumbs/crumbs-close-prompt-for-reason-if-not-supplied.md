---
id: cr-by7
title: 'crumbs close: prompt for --reason if not supplied'
status: open
type: feature
priority: 2
tags:
- cli
- ux
created: 2026-04-04
updated: 2026-04-05
dependencies: []
phase: ''
---

# crumbs close: prompt for --reason if not supplied

The --reason flag on crumbs close is optional and is frequently omitted, leaving closed_reason empty. When closing interactively (not in a script / --no-interactive context), prompt the user for a reason if none is provided. Improves traceability without being disruptive. Should be skippable with Enter for speed and suppressible with a --no-prompt / --yes flag for scripted use.
