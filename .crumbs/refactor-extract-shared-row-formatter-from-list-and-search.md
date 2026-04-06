---
id: cr-e3q
title: 'Refactor: extract shared row-formatter from list and search'
status: in_progress
type: feature
priority: 2
tags:
- cli
- refactor
created: 2026-04-06
updated: 2026-04-06
dependencies: []
phase: ''
---

# Refactor: extract shared row-formatter from list and search

Extracted format_row into commands/row.rs; list.rs and search.rs now delegate to it. PR #21 open for review.
