---
id: cr-i4i
title: Markdown editor autosave after delay
status: closed
type: task
priority: 2
tags: []
created: 2026-03-07
updated: 2026-03-07
closed_reason: Implemented debounced autosave (2s) on input, immediate flush on blur and Cmd/Ctrl-S.
dependencies: []
description: |-
  Right now the markdown editor only seems to autosave when focus is changed to another crumb. Which means one has to click twice on the other crumb to switch over after making edits. Save after (say) 2 seconds of inaction may help with this.

  Also, adding a save keyboard shortcut might help. Cmd/Ctrl-S is the natural choce.
---

# Markdown editor autosave after delay

Right now the markdown editor only seems to autosave when focus is changed to another crumb. Which means one has to click twice on the other crumb to switch over after making edits. Save after (say) 2 seconds of inaction may help with this.

Also, adding a save keyboard shortcut might help. Cmd/Ctrl-S is the natural choce.
