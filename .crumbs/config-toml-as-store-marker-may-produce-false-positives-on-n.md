---
id: cr-nmx
title: config.toml as store marker may produce false positives on non-crumbs directories
status: open
type: task
priority: 2
tags: []
created: 2026-07-07
updated: 2026-07-07
phase: ''
---

# config.toml as store marker may produce false positives on non-crumbs directories

has_store() and to_path() both treat config.toml as a store marker file. This is a legacy concern — config.toml is migrated to crumbs.toml on first store access. However, config.toml is also a common filename in Rust projects, so a project root with a non-crumbs config.toml could be incorrectly identified as a store. Low practical impact (the migration path makes this transient), but once the config.toml→crumbs.toml migration is fully rolled out in production, remove config.toml from the marker list in has_store and to_path to eliminate the false-positive surface.
