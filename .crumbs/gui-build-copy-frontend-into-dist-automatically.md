---
id: cr-idg
title: 'GUI build: copy frontend into dist/ automatically'
status: closed
type: task
priority: 2
tags:
- gui
- build
created: 2026-11-13
updated: 2026-11-13
closed_reason: 'Done: frontend served from crumbs-gui/web/ (frontendDist), no copy step; gui-install/dist replace the .app bundle. Node pin in CI skipped: runner Node 20+ suffices for the node:test step.'
phase: ''
---

# GUI build: copy frontend into dist/ automatically

Releases (tauri-action) build from the committed crumbs-gui/dist/, which drifts from the sources; the just recipes repeat the cp list in four places. Add a beforeBuildCommand/beforeDevCommand in tauri.conf.json (or a single _sync-dist recipe plus a release-workflow step) so dist/ is always rebuilt from source. Also consider pinning Node via actions/setup-node for the GUI unit-test CI step. From cr-lvq review, 2026-11-13.

[2026-11-13] `just gui-install` uses `cp -r ... /Applications/crumbs.app`; when the app already exists this nests the new bundle inside the old one (found crumbs.app/crumbs.app, outer app stuck at 0.25.2). Replace the bundle instead (rm -rf then cp -R, or ditto into a fresh path).

[start] 2026-11-13 21:45:43  Sync dist/ via Tauri before-commands

[2026-11-13] Plan: (1) crumbs-gui/sync-dist.js (node, portable across macOS/Linux/Windows runners) copies index.html, main.js, date.js, style.css into dist/; node:test coverage. (2) tauri.conf.json beforeBuildCommand + beforeDevCommand (wait) run it, so just recipes, plain cargo tauri dev/build and tauri-action all sync. (3) Untrack the four synced dist/ copies and gitignore them; codemirror.bundle.js and marked.min.js stay committed. (4) justfile: drop the four cp lines; gui-install replaces /Applications/crumbs.app instead of nesting. (5) Node pin in CI skipped: runner Node 20+ suffices and release builds rely on runner Node anyway. Patch bump.

[stop]  2026-11-13 22:00:32  14m 49s
