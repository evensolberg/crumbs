---
id: cr-iiw
title: 'CI: run crumbs-gui unit tests'
status: open
type: task
priority: 2
tags:
- ci
- gui
created: 2026-11-13
updated: 2026-11-13
phase: ''
---

# CI: run crumbs-gui unit tests

ci.yml only runs `cargo check -p crumbs-gui`, so the 15 unit tests in crumbs-gui/src-tauri/src/commands.rs (to_path, has_store, etc.) never run in CI. Add `cargo test -p crumbs-gui` (or nextest) after the Linux GUI deps step. Found while fixing cr-nmx, 2026-11-13.
