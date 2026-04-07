---
id: cr-w8z
title: Add pr / commit field for linking closed crumbs to their resolution
status: closed
type: feature
priority: 2
tags:
- cli
- metadata
created: 2026-04-04
updated: 2026-04-06
dependencies: []
phase: ''
---

# Add pr / commit field for linking closed crumbs to their resolution

closed_reason is freetext.

Structuring the resolution reference as a typed field (pr: 42 or commit: abc1234) would allow tooling to generate changelogs, link back to GitHub PRs, and verify that referenced PRs are actually merged. Could be rendered as a clickable link in the GUI.

The freetext closed_reason field would remain for narrative explanation.

[start] 2026-04-06 16:23:01  Adding resolution field (pr/commit) to Item; CLI update + GUI detail pane

[stop]  2026-04-06 21:11:17  4h 48m 16s  Merged PR #23
