---
id: cr-tng
title: 'Fix: description param in create::run should be &str not String'
status: closed
type: task
priority: 3
tags:
- cli
- lint
created: 2026-03-29
updated: 2026-03-31
closed_reason: 'Resolved via CreateArgs struct in PR #8'
dependencies: []
---

# Fix: description param in create::run should be &str not String

create::run takes description: String but does not consume it (needless_pass_by_value). Change to &str. Blocked on the CreateArgs refactor (see companion crumb) since fixing the signature requires coordinated updates to main.rs, integration tests, and the GUI Tauri command handler.

[start] 2026-03-30 21:31:20  Tackling alongside cr-bvl (CreateArgs refactor) — both touch create::run

[stop]  2026-03-30 21:45:47  14m 27s  CreateArgs struct resolves needless_pass_by_value; description is now owned by the struct
