---
id: cr-26i
title: update —message
status: closed
type: bug
priority: 2
tags: []
created: 2026-08-27
updated: 2026-11-13
closed_reason: 'Wontfix: --message replace semantics are intended; use --append to add notes'
phase: ''
---

# update —message

Using the `crumbs update` with the `--message` option overwrites the old message with just the update. We need to prevent `--mesage` from being available with `update`.
