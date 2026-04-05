---
id: cr-d23
title: Consider adding a TUI editor to the CLI version
status: closed
type: task
priority: 2
tags: []
created: 2026-03-09
updated: 2026-03-11
closed_reason: ''
dependencies: []
---

# Consider adding a TUI editor to the CLI version

Ratatui? Something else? Open to suggestions.Good

[2026-03-10] ratatui-textarea (v0.8.0) is the right building block — it provides cursor, undo/redo, search out of the box on top of ratatui. Full split-pane TUI (properties + editor) is high effort for modest gain given $EDITOR already works well for heavy edits. Narrower scope worth considering: crumbs note <id> — opens just the body inline via ratatui-textarea. Covers the main gap at ~20% of the effort.

[start] 2026-03-10 13:17:59

[stop]  2026-03-11 00:10:21  10h 52m 22s
