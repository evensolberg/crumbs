# GUI Multi-Select and Bulk Edit — Design Spec

**Date:** 2026-04-11
**Crumb:** cr-nk8
**Status:** Approved

---

## Overview

Add shift-click and Cmd/Ctrl-click multi-selection to the GUI item table, with a bulk-edit panel that appears in the detail pane when more than one item is selected. All bulk operations call existing single-ID Tauri commands sequentially from JS — no new Rust code required.

---

## 1. Selection Model

### State

Replace the single `selectedId: string | null` with a `selectedIds: Set<string>`. Single-select behaviour is preserved — a plain click clears the set and adds one ID, identical to today.

### Gestures

| Gesture | Behaviour |
| --- | --- |
| Click | Clear selection, select that row (existing behaviour) |
| Cmd/Ctrl+click | Toggle the clicked row in/out of `selectedIds` |
| Shift+click | Range-select from the last-clicked row to the clicked row (inclusive), using the current filtered row order |
| Cmd+A | Select all rows in the current filtered/search result set, regardless of scroll position |
| Escape | Clear multi-selection entirely |
| ↑ / ↓ | Navigate rows (existing behaviour, unchanged) |
| Delete / Backspace | If `selectedIds.size > 1`, open bulk delete confirmation; otherwise existing single-delete behaviour |

### Visual Feedback

Selected rows receive the existing `.selected` CSS class. Multiple rows can carry the class simultaneously — no CSS changes required.

---

## 2. Bulk-Edit Panel

When `selectedIds.size > 1`, `renderDetail()` renders a bulk-edit panel instead of the single-item detail view. When the selection drops back to 0 or 1, normal behaviour resumes.

### Panel Fields

| Field | Control | Behaviour |
| --- | --- | --- |
| Header | `"N items selected"` label | Always shown |
| Status | `<select>` | Options: open / in_progress / blocked / deferred / closed. Shows `— mixed —` when values differ across selection. Choosing "closed" triggers the close modal (see §3). |
| Priority | `<select>` | P1 / P2 / P3 / P4 + `— mixed —` |
| Type | `<select>` | feature / bug / task / idea / epic + `— mixed —` |
| Due date | `<input type="date">` | Blank when values differ |
| Add tags | `<input type="text">` | Comma-separated IDs; merged into each item's existing tags |
| Replace tags | `<input type="text">` | Comma-separated IDs; overwrites all tags on every selected item |
| Add blocker | `<input type="text">` | Single crumb ID; added as `blocked_by` on all selected items via `link_items` |
| Remove blocker | `<input type="text">` | Single crumb ID; removed from `blocked_by` on all selected items via `link_items` |
| **Apply** | `<button>` | Applies only changed fields (skips `— mixed —` / blank); see §4 |
| **Delete all** | `<button>` (destructive) | Opens delete confirmation modal; deletes all selected items |

> **Note:** `depends` is intentionally excluded. It expresses the same relationship as `blocked_by` but is unidirectional and has no status semantics. Removing `depends` from the data model entirely is tracked in **cr-kwh**.

---

## 3. Closing Items in Bulk

Selecting "closed" from the Status dropdown opens the existing close confirmation modal once. The reason entered applies to every item in `selectedIds`. After confirmation, `close_item(id, reason)` is called sequentially for each ID.

---

## 4. Apply Logic

`applyBulk(ids, fn)` is a small JS helper that:

1. Iterates over `selectedIds`.
2. For each changed field, calls the corresponding existing Tauri command per ID.
3. Collects any per-item errors and surfaces them after the loop completes.
4. Calls `refreshList()` once at the end regardless of errors.

Fields left at `— mixed —` or blank are skipped entirely.

### Tauri commands used (all existing)

| Field | Command |
| --- | --- |
| Status | `update_status` |
| Priority | `update_priority` |
| Type | `update_type` |
| Due date | `update_due` |
| Add / Replace tags | `update_tags` (merge vs replace handled in JS before call) |
| Add / Remove blocker | `link_items` with `relation: 'blocked-by'`, `remove: false / true` |
| Close | `close_item` |
| Delete | `delete_item` |

No new Rust code is required.

---

## 5. Out of Scope

- Bulk editing of body text
- Bulk linking of `blocks` direction (add/remove items that the selection blocks)
- Batch Tauri commands in Rust
- Drag-and-drop of multi-selected rows between stores
- Keyboard extension of selection (Shift+↑/↓)

These may be revisited in a future iteration.

---

## 6. Files Affected

| File | Change |
| --- | --- |
| `crumbs-gui/main.js` | Selection state, gesture handlers, `renderDetail`, `applyBulk`, keyboard shortcuts |
| `crumbs-gui/index.html` | No changes expected |
| `crumbs-gui/style.css` | Minor tweaks if bulk panel needs distinct styling |
| `crumbs-gui/src-tauri/src/commands.rs` | No changes |
| `crumbs-gui/src-tauri/src/main.rs` | No changes |
