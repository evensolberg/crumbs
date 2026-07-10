---
id: cr-uuz
title: Pin ubuntu runner version in ci.yml and release.yml together
status: open
type: task
priority: 2
tags:
- ci
- mergify
created: 2026-07-09
updated: 2026-07-09
phase: ''
---

# Pin ubuntu runner version in ci.yml and release.yml together

Copilot review on PR #70 flagged that ci.yml's use of ubuntu-latest could break when GitHub updates the default image, especially combined with version-specific apt packages like libwebkit2gtk-4.1-dev. Decided not to fix in PR #70 because pinning only ci.yml would create drift from release.yml (which also uses ubuntu-latest unpinned) -- the fix needs to pin both workflows to the same Ubuntu version in lockstep so CI validates against the same OS the release build uses.
