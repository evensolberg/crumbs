---
id: cr-num
title: 'CLI: append to item body without opening editor'
status: closed
type: feature
priority: 2
tags:
- cli
- ux
created: 2026-03-07
updated: 2026-03-07
closed_reason: 'Implemented: --append flag on crumbs update appends a [YYYY-MM-DD]-timestamped note to the existing body instead of replacing it'
dependencies: []
---

# CLI: append to item body without opening editor

update --message replaces the body entirely. Add an --append flag (or a separate `crumbs note <id> <text>` command) to append a timestamped note to the body without touching the rest.

[2026-03-07] Also consider crumbs note <id> <text> as a shorthand.
