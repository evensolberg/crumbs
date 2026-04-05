---
id: cr-mvk
title: 'Document invariant: run_structured_commands only called with Create/List/Update'
status: closed
type: task
priority: 3
tags:
- lint
- refactor
created: 2026-03-31
updated: 2026-03-31
closed_reason: Comment added in a573613
dependencies: []
---

# Document invariant: run_structured_commands only called with Create/List/Update

The unreachable\!() catch-all in run_structured_commands is safe only because the sole caller (run_command) uses an @-binding restricting to exactly those three variants. If a new "structured" command is added, a dev must update BOTH the @-binding in run_command AND add an arm in run_structured_commands. Consider adding a comment documenting this invariant, or collapse the two functions once the line-count concerns are met.
