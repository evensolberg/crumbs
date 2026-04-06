---
id: cr-e3q
title: 'Refactor: extract shared row-formatter from list and search'
status: open
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

list.rs and search.rs duplicate the phase-width/badge/timer/tags/due/points rendering logic. Extract a shared helper (e.g. format_row) so future format changes only need to be made in one place.
