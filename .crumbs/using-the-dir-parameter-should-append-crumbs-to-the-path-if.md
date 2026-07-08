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

This bug is still present in the GUI client. The store path resolution in the Tauri backend did not append .crumbs to project root paths passed from the GUI frontend.
