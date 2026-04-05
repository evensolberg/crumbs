---
id: cr-bxa
title: 'CLI: batch create from file or stdin'
status: open
type: feature
priority: 2
tags:
- cli
- bulk
created: 2026-04-04
updated: 2026-04-04
dependencies: []
---

# CLI: batch create from file or stdin

No way to create multiple crumbs in one operation. When bootstrapping a backlog (e.g. from an audit or roadmap document), each crumb requires a separate command invocation. Add crumbs create --from <file.json|file.yaml> and/or crumbs create - (read JSON/YAML array from stdin). Complements cr-ghd (bulk operations on filtered items) which covers the update side.
