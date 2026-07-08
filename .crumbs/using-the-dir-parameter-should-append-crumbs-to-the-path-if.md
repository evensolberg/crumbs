---
id: cr-mya
title: Using the --dir parameter should append /.crumbs to the path if it is absent, otherwise things get created in the target directory and not the .crumbs subdirectory.
status: in_progress
type: bug
priority: 1
tags: []
created: 2026-03-07
updated: 2026-07-07
phase: ''
---

# Using the --dir parameter should append /.crumbs to the path if it is absent, otherwise things get created in the target directory and not the .crumbs subdirectory.

[2026-07-07] Reopened (was closed: Fixed: resolve_dir now appends .crumbs unless path already ends with .crumbs or contains store markers)

This bug is still present in the GUI client. See /Volumes/SSD/Souce/StratMan/ for an example.

[start] 2026-07-07 20:31:03  Investigating GUI-side resolve_dir fix
