---
id: cr-vaj
title: 'index.csv: embedded double-quotes not escaped per RFC4180'
status: open
type: bug
priority: 3
tags:
- csv
- index
- rfc4180
- quoting
created: 2026-07-02
updated: 2026-07-02
phase: ''
---

# index.csv: embedded double-quotes not escaped per RFC4180

When a crumb title contains double-quote characters (e.g. `.replace('"', "")`),
the generated `index.csv` wraps the field in quotes but does not double the
embedded quotes as RFC4180 requires. Any conformant CSV parser will mis-parse
the field — the title appears to terminate early at the first embedded quote.

Fix: when writing the title column, escape every `"` as `""` before wrapping
the field in outer double-quotes.

Discovered via Copilot review of `evensolberg/gather` PR #82 (`gtr-4h2` title
was the offending row).
