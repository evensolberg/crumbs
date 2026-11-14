---
id: cr-olr
title: Remove legacy config.toml migration once local stores are migrated
status: open
type: task
priority: 4
tags:
- config
- cleanup
created: 2026-11-13
updated: 2026-11-13
phase: ''
---

# Remove legacy config.toml migration once local stores are migrated

store_config::load still migrates a strict legacy config.toml (prefix only) to crumbs.toml. As of 2026-11-13 three local stores are unmigrated: ambient_tools, json2toon, docmeta (.crumbs/config.toml). Once they are opened (or migrated by hand), remove LegacyConfig, the rename and its tests, and add a changelog note. Removing it earlier would silently give those stores the default cr prefix. From the cr-nmx simplify pass.
