---
id: cr-udj
title: 'CLI: add clap::ValueEnum to SortKey for shell completions'
status: closed
type: feature
priority: 3
tags:
- cli
- completions
created: 2026-03-29
updated: 2026-03-31
closed_reason: 'Implemented in PR #10; ValueEnum on SortKey with infallible Display match'
dependencies: []
---

# CLI: add clap::ValueEnum to SortKey for shell completions

Currently SortKey uses a manual FromStr impl. Deriving clap::ValueEnum would restore shell completions for --sort (crumbs list --sort <TAB> would offer id, priority, status, etc.) and improve --help output. Requires either adding clap as a lib dependency or using a thin newtype in main.rs. Noted during code review of PR #3.

[start] 2026-03-31 20:49:30

[stop]  2026-03-31 20:56:57  7m 27s  PR #10 open, awaiting Copilot review
