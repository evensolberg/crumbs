---
id: cr-idg
title: 'GUI build: copy frontend into dist/ automatically'
status: open
type: task
priority: 2
tags:
- gui
- build
created: 2026-11-13
updated: 2026-11-13
phase: ''
---

# GUI build: copy frontend into dist/ automatically

Releases (tauri-action) build from the committed crumbs-gui/dist/, which drifts from the sources; the just recipes repeat the cp list in four places. Add a beforeBuildCommand/beforeDevCommand in tauri.conf.json (or a single _sync-dist recipe plus a release-workflow step) so dist/ is always rebuilt from source. Also consider pinning Node via actions/setup-node for the GUI unit-test CI step. From cr-lvq review, 2026-11-13.
