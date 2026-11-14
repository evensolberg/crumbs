---
id: cr-olr
title: Remove legacy config.toml migration once local stores are migrated
status: closed
type: task
priority: 4
tags:
- config
- cleanup
created: 2026-11-13
updated: 2026-11-14
closed_reason: 'Removed in PR #97 (v0.26.0) after migrating ambient_tools, docmeta and json2toon'
phase: ''
---

# Remove legacy config.toml migration once local stores are migrated

store_config::load still migrates a strict legacy config.toml (prefix only) to crumbs.toml. As of 2026-11-13 three local stores are unmigrated: ambient_tools, json2toon, docmeta (.crumbs/config.toml). Once they are opened (or migrated by hand), remove LegacyConfig, the rename and its tests, and add a changelog note. Removing it earlier would silently give those stores the default cr prefix. From the cr-nmx simplify pass.

[2026-11-14] ambient_tools, docmeta and json2toon migrated (config.toml renamed to crumbs.toml by hand; prefixes at/meta/j verified with a probe create). No local .crumbs/config.toml left under /Volumes/SSD/Source or ~. Unblocked.

[start] 2026-11-14 08:17:37

[stop]  2026-11-14 08:22:49  5m 12s
