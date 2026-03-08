---
id: cr-7c9
title: 'CLI: message starting with -- crashes create'
status: closed
type: bug
priority: 1
tags:
- cli
created: 2026-03-07
updated: 2026-03-07
closed_reason: 'Fixed: added allow_hyphen_values = true to message arg in Create, C, and Update commands'
dependencies: []
---

# CLI: message starting with -- crashes create

Using --message with a value that starts with -- causes clap to misparse it as an argument. E.g. crumbs create "title" --message "--foo is broken". Should be handled gracefully.
