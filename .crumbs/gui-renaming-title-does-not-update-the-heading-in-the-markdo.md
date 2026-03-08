---
id: cr-h0k
title: 'GUI: renaming title does not update the heading in the markdown body'
status: closed
type: bug
priority: 2
tags:
- gui
- title
created: 2026-03-07
updated: 2026-03-07
closed_reason: 'Fixed: update.rs now extracts existing description and rebuilds heading from item.title, so any title rename is reflected in the body heading'
dependencies: []
---

# GUI: renaming title does not update the heading in the markdown body

When a title is renamed via the GUI — either via the inline edit in the detail pane title label (double-click) or the table row rename — update_title rewrites the frontmatter but leaves the # heading in the markdown body unchanged. This causes a heading/title mismatch warning on every parse, and the description may be silently dropped. Fix: update.rs should rewrite the heading line in the body whenever the title changes.
