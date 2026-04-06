---
id: cr-e3q
title: 'Refactor: extract shared row-formatter from list and search'
status: closed
type: feature
priority: 2
tags:
- cli
- refactor
created: 2026-04-06
updated: 2026-04-06
closed_reason: 'Extracted format_row + PhaseColumn into commands/row.rs; merged in PR #21 (v0.19.1)'
dependencies: []
phase: ''
---

# Refactor: extract shared row-formatter from list and search

Extracted format_row into commands/row.rs; list.rs and search.rs now delegate to it. PR #21 open for review.
