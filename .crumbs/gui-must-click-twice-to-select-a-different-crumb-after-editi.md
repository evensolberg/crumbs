---
id: cr-ydo
title: 'GUI: must click twice to select a different crumb after editing body'
status: closed
type: bug
priority: 1
tags:
- gui
- autosave
created: 2026-03-07
updated: 2026-03-07
closed_reason: 'Fixed as part of cr-j90: skipping the no-op blur save eliminates the spurious loadItems re-render that ate the first click'
dependencies: []
---

# GUI: must click twice to select a different crumb after editing body

After editing the body text and the cursor is still in the textarea, clicking another row in the table requires two clicks: the first blur triggers flushAutosave (which calls loadItems and re-renders the table), and the row-click event is lost in the re-render. Fix is related to cr-j90 — skip the save on blur if content is unchanged, eliminating the spurious reload.
