---
id: cr-bvl
title: 'Refactor: create::run has too many arguments (9/7)'
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

# Refactor: create::run has too many arguments (9/7)

create::run in crumbs/src/commands/create.rs takes 9 arguments, exceeding the clippy::too_many_arguments limit of 7. Bundle into a CreateArgs struct matching the UpdateArgs/ListArgs pattern. Requires updating callers in main.rs, tests, and the GUI Tauri command.

[start] 2026-03-30 21:31:20  Bundling with cr-tng; CreateArgs struct resolves both needless_pass_by_value and too_many_arguments

[stop]  2026-03-30 21:45:47  14m 27s  CreateArgs struct reduces create::run to 2 args; PR #8
