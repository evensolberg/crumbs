---
id: cr-w8z
title: Add pr / commit field for linking closed crumbs to their resolution
status: open
type: feature
priority: 3
tags:
- cli
- metadata
created: 2026-04-04
updated: 2026-04-04
dependencies: []
---

# Add pr / commit field for linking closed crumbs to their resolution

closed_reason is freetext. Structuring the resolution reference as a typed field (pr: 42 or commit: abc1234) would allow tooling to generate changelogs, link back to GitHub PRs, and verify that referenced PRs are actually merged. Could be rendered as a clickable link in the GUI. The freetext closed_reason field would remain for narrative explanation.
