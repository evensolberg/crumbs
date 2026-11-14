---
id: cr-sfr
title: GUI hard-codes store marker file names owned by the library
status: open
type: task
priority: 4
tags:
- gui
- config
- cleanup
created: 2026-11-14
updated: 2026-11-14
phase: ''
---

# GUI hard-codes store marker file names owned by the library

has_marker_files in crumbs-gui/src-tauri/src/commands.rs hard-codes "index.csv" and "crumbs.toml", while store_config::CONFIG_FILE is private. A future rename would need a matching GUI edit and could drift silently. Expose a pub is_store_dir(&Path) (or pub CONFIG_FILE) from the library and call it. save() also hard-codes "write crumbs.toml" in its context message. From the cr-olr simplify pass.
