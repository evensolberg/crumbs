---
id: cr-n4x
title: Monitor serde_toon for enum round-trip support
status: open
type: feature
priority: 4
tags:
- cli
- gui
- import
created: 2026-04-08
updated: 2026-04-08
dependencies: []
phase: ''
---

# Monitor serde_toon for enum round-trip support

serde_toon currently cannot round-trip enum variants (e.g. Status::Open) via from_slice — it serialises them as bare strings but the serde-derived Deserialize impl rejects visit_str. This blocks TOON support in 'crumbs import --file' and the GUI Import button. Periodically check serde_toon_format crate for updates or a workaround. When fixed: add 'toon' to file_import::infer_format and parse_items, add .toon to the GUI file picker filter, add end-to-end TOON import test, update skill and README.
