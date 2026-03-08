---
id: cr-ogw
title: 'CLI: defer with an until date'
status: closed
type: feature
priority: 3
tags:
- cli
- defer
created: 2026-03-07
updated: 2026-03-07
closed_reason: 'CLI: defer --until <date> sets due date; next skips deferred items with future due. GUI: Defer button opens modal with optional date picker; doNext aligned with CLI logic'
dependencies: []
---

# CLI: defer with an until date

defer sets status to deferred but carries no date. Add --until <date> so the item surfaces again (e.g. in `crumbs next`) after that date, combining deferral with due-date semantics.
