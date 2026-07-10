---
id: cr-o2e
title: Extract shared Linux GUI build deps into a composite action
status: open
type: task
priority: 2
tags:
- ci
- mergify
created: 2026-07-09
updated: 2026-07-09
phase: ''
---

# Extract shared Linux GUI build deps into a composite action

Simplify pass on PR #70 flagged that ci.yml and release.yml both independently declare the same apt-get install list (libwebkit2gtk-4.1-dev, libappindicator3-dev, librsvg2-dev, patchelf) for building crumbs-gui on Linux. This is an exact duplicate tied to the Tauri/webkit2gtk version -- if it ever needs to change, both files must be updated in lockstep or CI and release builds silently diverge. Deferred because fixing it means touching release.yml, which is outside PR #70's scope (dependency automation). Extract into a composite action, e.g. .github/actions/install-gui-linux-deps/action.yml, and have both workflows call it. Consider bundling with cr-uuz (ubuntu runner pinning) since both are about keeping ci.yml and release.yml's Linux/GUI build environment in sync.
