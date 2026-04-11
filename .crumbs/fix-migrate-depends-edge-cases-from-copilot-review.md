---
id: cr-zp2
title: Fix migrate_depends edge cases from Copilot review
status: closed
type: task
priority: 2
tags:
- bug
- store
- tests
created: 2026-04-11
updated: 2026-04-11
phase: ''
---

# Fix migrate_depends edge cases from Copilot review

Three issues raised in second Copilot review of PR #37 (merged):
1. No guard for blank dep_id -- empty string pushed into blocked_by
2. No self-cycle guard -- item depending on itself creates invalid graph
3. Test setup: create_dir_all before init::run causes init to short-circuit, config.toml never written (three tests)
4. Inline comment in unknown-ID test still says 'silently ignored'

[start] 2026-04-11 14:19:22

[stop]  2026-04-11 14:28:44  9m 22s
