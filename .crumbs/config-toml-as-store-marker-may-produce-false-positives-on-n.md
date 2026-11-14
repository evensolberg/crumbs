---
id: cr-nmx
title: config.toml as store marker may produce false positives on non-crumbs directories
status: closed
type: task
priority: 2
tags: []
created: 2026-07-07
updated: 2026-11-13
closed_reason: 'Fixed: GUI no longer treats config.toml as a store marker; store_config::load only migrates a strict legacy crumbs config and warns otherwise. Removal of the migration deferred to cr-olr.'
phase: ''
---

# config.toml as store marker may produce false positives on non-crumbs directories

has_store() and to_path() both treat config.toml as a store marker file. This is a legacy concern — config.toml is migrated to crumbs.toml on first store access. However, config.toml is also a common filename in Rust projects, so a project root with a non-crumbs config.toml could be incorrectly identified as a store. Low practical impact (the migration path makes this transient), but once the config.toml→crumbs.toml migration is fully rolled out in production, remove config.toml from the marker list in has_store and to_path to eliminate the false-positive surface.

[start] 2026-11-13 22:22:26  Drop config.toml from GUI store markers

[2026-11-13] Plan: remove config.toml from has_marker_files (crumbs-gui commands.rs), used by to_path and has_store. Worse than described: a project root with its own config.toml and no .crumbs/ is taken as a flat store, and store_config::load then renames that config.toml to crumbs.toml. Keep the CLI migration in store_config::load: 3 local stores (ambient_tools, json2toon, docmeta) are still unmigrated, but all are found via the .crumbs suffix/subdir checks that run before markers. Flip the two tests that assert config.toml is a marker into regression tests. Patch bump (observable GUI fix).

[stop]  2026-11-13 22:29:45  7m 19s
