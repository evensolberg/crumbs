---
id: cr-mop
title: 'Refactor: split expand_shortcodes() to reduce line count'
status: closed
type: task
priority: 3
tags:
- cli
- lint
created: 2026-03-29
updated: 2026-03-31
closed_reason: 'Resolved in PR #9'
dependencies: []
---

# Refactor: split expand_shortcodes() to reduce line count

expand_shortcodes() in crumbs/src/emoji.rs is 107 lines (limit 100). Extract the inner scanning loop into a named helper to bring it under the clippy::too_many_lines threshold.

[start] 2026-03-31 07:57:26  Splitting alongside cr-rgl; one PR for both too_many_lines fixes

[stop]  2026-03-31 20:45:36  12h 48m 10s  Done via PR #9
