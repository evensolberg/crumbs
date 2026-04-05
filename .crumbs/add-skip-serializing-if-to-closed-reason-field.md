---
id: cr-wks
title: Add skip_serializing_if to closed_reason field
status: closed
type: bug
priority: 3
tags:
- refactor
created: 2026-03-31
updated: 2026-03-31
closed_reason: 'Fixed in PR #11 alongside cr-47s'
dependencies: []
---

# Add skip_serializing_if to closed_reason field

closed_reason in item.rs lacks #[serde(skip_serializing_if = "String::is_empty")] unlike the description field. After reopening a crumb, the frontmatter contains closed_reason: "" rather than omitting it. Fix: add the attribute to Item::closed_reason in item.rs.
