# GUI Multi-Select and Bulk Edit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Cmd/Shift-click multi-selection to the item table and a bulk-edit panel in the detail pane for status, priority, type, due, tags, and blockers.

**Architecture:** All changes are confined to `crumbs-gui/main.js` (and minor CSS in `style.css`). The single `selectedId: string | null` variable is replaced with `selectedIds: Set<string>` plus a `primaryId()` helper that preserves single-select behaviour for all existing code paths. When `selectedIds.size > 1`, `renderDetail()` delegates to a new `renderBulkPanel()` that hides the editor column and renders a bulk-edit form in the properties column. Bulk operations call existing single-ID Tauri commands sequentially via an `applyBulk()` helper — no Rust changes required.

**Tech Stack:** Vanilla JS, Tauri 2 `invoke`, existing CSS custom properties in `style.css`. No JS test framework exists in this project; each task includes manual verification steps instead of automated tests.

**Spec:** `docs/specs/2026-04-11-gui-multi-select-edit-design.md`

---

## File Map

| File | What changes |
|---|---|
| `crumbs-gui/main.js` | All logic: state, gestures, keyboard, panel rendering, apply/close/delete |
| `crumbs-gui/style.css` | `.bulk-panel`, `.bulk-row`, `.bulk-label`, `.btn-danger` styles; `detail-pane.bulk-mode` layout |
| `crumbs-gui/index.html` | Add multi-select shortcuts to help modal (Task 8) |
| `crumbs-gui/src-tauri/src/` | No changes |

---

## Task 1: Replace `selectedId` with `selectedIds` Set

**Files:**
- Modify: `crumbs-gui/main.js`

> No JS test framework — verify manually at end of task by opening the GUI and confirming single-click selection still works.

- [ ] **Step 1: Replace the `selectedId` declaration (line 30)**

Find:
```js
let selectedId = null;
```
Replace with:
```js
let selectedIds    = new Set();   // all currently highlighted IDs
let lastClickedId  = null;        // anchor for shift-range selection
```

- [ ] **Step 2: Add `primaryId()` helper immediately after the new declarations**

Insert after the two new `let` lines:
```js
/** The single selected ID, or null when 0 or >1 rows are selected. */
function primaryId() {
  return selectedIds.size === 1 ? [...selectedIds][0] : null;
}
```

- [ ] **Step 3: Update `selectedItem()` (around line 341)**

Find:
```js
function selectedItem() {
  return allItems.find(i => i.id === selectedId) ?? null;
}
```
Replace with:
```js
function selectedItem() {
  const id = primaryId();
  return id ? (allItems.find(i => i.id === id) ?? null) : null;
}
```

- [ ] **Step 4: Update `selectRow()` (around line 369)**

Find:
```js
function selectRow(id, tr) {
  selectedId = id;
  for (const r of document.querySelectorAll('#items-body tr.selected')) r.classList.remove('selected');
  if (tr) tr.classList.add('selected');
  renderDetail(selectedItem());
}
```
Replace with:
```js
function selectRow(id, tr) {
  selectedIds.clear();
  if (id != null) selectedIds.add(id);
  lastClickedId = id ?? null;
  for (const r of document.querySelectorAll('#items-body tr.selected')) r.classList.remove('selected');
  if (tr) tr.classList.add('selected');
  renderDetail(selectedItem());
}
```

- [ ] **Step 5: Update `renderTable()` row-highlight (around line 626)**

Find:
```js
if (item.id === selectedId) tr.classList.add('selected');
```
Replace with:
```js
if (selectedIds.has(item.id)) tr.classList.add('selected');
```

- [ ] **Step 6: Update `updateToolbarButtons()` (around line 393)**

Find:
```js
const item = selectedItem();
const hasSelection = item !== null;
```
Replace with:
```js
const item = selectedItem();
const hasSelection = selectedIds.size > 0;
```
Also update the `startBtn`/`blockBtn`/`deferBtn`/`timerBtn`/`closeItemBtn`/`emojiBtn` disabled expressions — they reference `item` which is now `null` for multi-select, so they naturally disable when `selectedIds.size > 1`. `deleteBtn` only needs `hasSelection`:

Find:
```js
deleteBtn.disabled   = !hasSelection;
emojiBtn.disabled    = !hasSelection;
```
Replace with:
```js
deleteBtn.disabled   = !hasSelection;
emojiBtn.disabled    = !hasSelection || selectedIds.size > 1;
```

- [ ] **Step 7: Update the arrow-key navigation handler (around line 2115)**

Find:
```js
const currentIndex = rows.findIndex(r => r.dataset.id === selectedId);
```
Replace with:
```js
const currentIndex = rows.findIndex(r => r.dataset.id === (lastClickedId ?? primaryId()));
```

- [ ] **Step 8: Update the Enter-key handler (around line 2127)**

Find:
```js
if (e.key === 'Enter' && selectedId) {
```
Replace with:
```js
if (e.key === 'Enter' && primaryId()) {
```

- [ ] **Step 9: Update the Delete/Backspace handler (around line 2135)**

Find:
```js
if ((e.key === 'Delete' || e.key === 'Backspace') && selectedId) {
```
Replace with:
```js
if ((e.key === 'Delete' || e.key === 'Backspace') && selectedIds.size > 0) {
```

- [ ] **Step 10: Replace all remaining `selectedId` read-accesses with `primaryId()`**

The following lines use `selectedId` as a read (not an assignment). Replace each:

| Location (approx line) | Find | Replace |
|---|---|---|
| `confirmDelete` ~1517 | `id: selectedId` | `id: primaryId()` |
| `confirmDelete` ~1518 | `selectedId = null;` | `selectedIds.clear(); lastClickedId = null;` |
| `confirmClose` ~1539 | `if (selectedId === pendingCloseId` | `if (primaryId() === pendingCloseId` |
| `confirmClose` ~1540 | `selectedId = null;` | `selectedIds.clear(); lastClickedId = null;` |
| `renderBlockerList` ~1556 | `i.id !== selectedId &&` | `i.id !== primaryId() &&` |
| `link_items` calls ~1594,1597 | `id: selectedId` | `id: primaryId()` |
| defer guard ~1653 | `if (!selectedId) return;` | `if (!primaryId()) return;` |
| `defer_item` ~1656 | `id: selectedId` | `id: primaryId()` |
| timer guard ~1676 | `if (!selectedId) return;` | `if (!primaryId()) return;` |
| timer guard ~1688 | `if (!selectedId) return;` | `if (!primaryId()) return;` |
| `start_timer`/`stop_timer` ~1695,1697 | `id: selectedId` | `id: primaryId()` |
| drag-drop handler ~1989 | `if (selectedId === id) selectedId = null;` | `if (selectedIds.has(id)) { selectedIds.delete(id); if (lastClickedId === id) lastClickedId = null; }` |
| `loadItems` initial select ~1776 | `selectedId = null;` | `selectedIds.clear(); lastClickedId = null;` |

- [ ] **Step 11: Verify single-select still works**

Run: `just gui-dev`

- Click any row — it highlights, detail pane shows the item's properties.
- Click a different row — first row deselects, new row highlights.
- ↑ / ↓ arrows navigate as before.
- No console errors.

- [ ] **Step 12: Commit**

```bash
git mit es
git add crumbs-gui/main.js
git commit -m "Refactor: replace selectedId with selectedIds Set + primaryId() helper"
```

---

## Task 2: Add `updateRowHighlights()` helper + multi-select click gestures

**Files:**
- Modify: `crumbs-gui/main.js`

- [ ] **Step 1: Add `updateRowHighlights()` helper**

Insert this function directly below `selectRow()`:
```js
/** Sync `.selected` class on all table rows to match `selectedIds`. */
function updateRowHighlights() {
  for (const r of document.querySelectorAll('#items-body tr[data-id]')) {
    r.classList.toggle('selected', selectedIds.has(r.dataset.id));
  }
}
```

- [ ] **Step 2: Replace the `itemsBody` click handler (around line 2305)**

Find:
```js
itemsBody.addEventListener('click', e => {
  const tr = e.target.closest('tr[data-id]');
  if (!tr) return;
  selectRow(tr.dataset.id, tr);
});
```
Replace with:
```js
itemsBody.addEventListener('click', e => {
  const tr = e.target.closest('tr[data-id]');
  if (!tr) return;
  const id = tr.dataset.id;

  if (e.metaKey || e.ctrlKey) {
    // Cmd/Ctrl+click: toggle this row in or out of the selection
    if (selectedIds.has(id)) {
      selectedIds.delete(id);
    } else {
      selectedIds.add(id);
    }
    lastClickedId = id;
    updateRowHighlights();
    renderDetail(selectedIds.size > 1 ? null : selectedItem());
    updateToolbarButtons();
    return;
  }

  if (e.shiftKey && lastClickedId) {
    // Shift+click: range-select from lastClickedId to this row (inclusive)
    const rows = [...document.querySelectorAll('#items-body tr[data-id]')];
    const anchorIdx = rows.findIndex(r => r.dataset.id === lastClickedId);
    const clickIdx  = rows.findIndex(r => r.dataset.id === id);
    if (anchorIdx !== -1 && clickIdx !== -1) {
      const [from, to] = anchorIdx <= clickIdx
        ? [anchorIdx, clickIdx]
        : [clickIdx, anchorIdx];
      selectedIds.clear();
      for (let i = from; i <= to; i++) selectedIds.add(rows[i].dataset.id);
      updateRowHighlights();
      renderDetail(selectedIds.size > 1 ? null : selectedItem());
      updateToolbarButtons();
      return;
    }
  }

  // Plain click: single-select
  selectRow(id, tr);
});
```

- [ ] **Step 3: Update the Escape handler (around line 2071) to also clear multi-selection**

Find:
```js
  if (e.key === 'Escape') {
    hideContextMenu();
    if (!helpModal.classList.contains('hidden')) {
      helpModal.classList.add('hidden');
    }
    return;
  }
```
Replace with:
```js
  if (e.key === 'Escape') {
    hideContextMenu();
    if (!helpModal.classList.contains('hidden')) {
      helpModal.classList.add('hidden');
      return;
    }
    if (selectedIds.size > 1) {
      selectedIds.clear();
      lastClickedId = null;
      updateRowHighlights();
      renderDetail(selectedItem());
      updateToolbarButtons();
      return;
    }
    return;
  }
```

- [ ] **Step 4: Verify multi-select gestures**

Run: `just gui-dev`

- Cmd+click two rows → both highlight; detail pane does not open (it renders nothing yet — that's fine, handled in Task 4).
- Shift+click a range → all rows between anchor and click highlight.
- Plain click after multi-select → collapses to single selection, detail pane reopens.
- Escape with multi-select → clears all highlights.

- [ ] **Step 5: Commit**

```bash
git mit es
git add crumbs-gui/main.js
git commit -m "Feat: add Cmd+click toggle and Shift+click range selection"
```

---

## Task 3: Cmd+A select-all and Delete/Backspace bulk upgrade

**Files:**
- Modify: `crumbs-gui/main.js`

- [ ] **Step 1: Add Cmd+A handler**

In the `document.addEventListener('keydown', ...)` block, insert after the `Cmd+R` block (around line 2104) and before the `isControlFocused` guard:

```js
  // Cmd/Ctrl+A — select all filtered rows
  if (mod && e.key === 'a' && !isControlFocused() && !isModalOpen()) {
    e.preventDefault();
    const rows = [...document.querySelectorAll('#items-body tr[data-id]')];
    selectedIds.clear();
    rows.forEach(r => selectedIds.add(r.dataset.id));
    lastClickedId = rows.length ? rows[rows.length - 1].dataset.id : null;
    updateRowHighlights();
    renderDetail(selectedIds.size > 1 ? null : selectedItem());
    updateToolbarButtons();
    return;
  }
```

- [ ] **Step 2: Update the Delete/Backspace handler to open bulk-delete for multi-select**

The handler was updated in Task 1 Step 9 to fire when `selectedIds.size > 0`. Now split it into single vs. bulk:

Find:
```js
  if ((e.key === 'Delete' || e.key === 'Backspace') && selectedIds.size > 0) {
    e.preventDefault();
    openDeleteModal();
    return;
  }
```
Replace with:
```js
  if ((e.key === 'Delete' || e.key === 'Backspace') && selectedIds.size > 0) {
    e.preventDefault();
    if (selectedIds.size > 1) {
      openBulkDeleteModal([...selectedIds]);
    } else {
      openDeleteModal();
    }
    return;
  }
```

- [ ] **Step 3: Verify**

Run: `just gui-dev`

- Cmd+A → all visible rows highlight.
- Cmd+A after filtering to a subset → only filtered rows highlight.
- Escape clears all.
- Single-select + Delete → existing delete modal still opens.
- Multi-select + Delete → will call `openBulkDeleteModal` (not yet defined — expect a console error; that's fine, bulk delete is Task 7).

- [ ] **Step 4: Commit**

```bash
git mit es
git add crumbs-gui/main.js
git commit -m "Feat: add Cmd+A select-all and bulk Delete/Backspace trigger"
```

---

## Task 4: `renderBulkPanel()` and layout

**Files:**
- Modify: `crumbs-gui/main.js`
- Modify: `crumbs-gui/style.css`

- [ ] **Step 1: Add `detailRight` constant**

Near the top of `main.js` where the other `const` element lookups are (around lines 100–117), add:
```js
const detailRight      = document.getElementById('detail-right');
```

- [ ] **Step 2: Update `renderDetail()` to branch on multi-select (around line 1229)**

Find the opening of `renderDetail`:
```js
function renderDetail(item) {
  if (!item) {
    detailPane.classList.add('hidden');
```
Replace the first two lines with:
```js
function renderDetail(item) {
  // Multi-select: hand off to bulk panel, then return
  if (selectedIds.size > 1) {
    renderBulkPanel([...selectedIds]);
    return;
  }

  // Restore normal layout if coming back from bulk mode
  detailPane.classList.remove('bulk-mode');
  detailRight.classList.remove('hidden');

  if (!item) {
    detailPane.classList.add('hidden');
```

- [ ] **Step 3: Write `renderBulkPanel(ids)`**

Add this function directly before `renderDetail`:

```js
function renderBulkPanel(ids) {
  const items = ids.map(id => allItems.find(i => i.id === id)).filter(Boolean);

  detailPane.classList.remove('hidden');
  detailPane.classList.add('bulk-mode');
  detailRight.classList.add('hidden');

  // Detect mixed values across the selection
  const uniq = (arr) => [...new Set(arr)];
  const statuses   = uniq(items.map(i => i.status));
  const priorities = uniq(items.map(i => String(i.priority)));
  const types      = uniq(items.map(i => i.type ?? ''));

  const MIXED = '';   // empty string = "— mixed —" placeholder option

  const statusVal   = statuses.length   === 1 ? statuses[0]   : MIXED;
  const priorityVal = priorities.length === 1 ? priorities[0] : MIXED;
  const typeVal     = types.length      === 1 ? types[0]      : MIXED;

  const mixedOpt = `<option value="" ${!statusVal ? 'selected' : ''}>— mixed —</option>`;

  propGrid.innerHTML = '';
  detailActions.innerHTML = `
    <div class="bulk-panel">
      <div class="bulk-header">${ids.length} items selected</div>

      <div class="bulk-row">
        <label class="bulk-label" for="bulk-status">Status</label>
        <select id="bulk-status" class="bulk-select">
          ${mixedOpt}
          <option value="open"        ${statusVal === 'open'        ? 'selected' : ''}>open</option>
          <option value="in_progress" ${statusVal === 'in_progress' ? 'selected' : ''}>in progress</option>
          <option value="blocked"     ${statusVal === 'blocked'     ? 'selected' : ''}>blocked</option>
          <option value="deferred"    ${statusVal === 'deferred'    ? 'selected' : ''}>deferred</option>
          <option value="closed"      ${statusVal === 'closed'      ? 'selected' : ''}>closed</option>
        </select>
      </div>

      <div class="bulk-row">
        <label class="bulk-label" for="bulk-priority">Priority</label>
        <select id="bulk-priority" class="bulk-select">
          <option value="" ${!priorityVal ? 'selected' : ''}>— mixed —</option>
          <option value="1" ${priorityVal === '1' ? 'selected' : ''}>P1</option>
          <option value="2" ${priorityVal === '2' ? 'selected' : ''}>P2</option>
          <option value="3" ${priorityVal === '3' ? 'selected' : ''}>P3</option>
          <option value="4" ${priorityVal === '4' ? 'selected' : ''}>P4</option>
        </select>
      </div>

      <div class="bulk-row">
        <label class="bulk-label" for="bulk-type">Type</label>
        <select id="bulk-type" class="bulk-select">
          <option value="" ${!typeVal ? 'selected' : ''}>— mixed —</option>
          <option value="feature" ${typeVal === 'feature' ? 'selected' : ''}>feature</option>
          <option value="bug"     ${typeVal === 'bug'     ? 'selected' : ''}>bug</option>
          <option value="task"    ${typeVal === 'task'    ? 'selected' : ''}>task</option>
          <option value="idea"    ${typeVal === 'idea'    ? 'selected' : ''}>idea</option>
          <option value="epic"    ${typeVal === 'epic'    ? 'selected' : ''}>epic</option>
        </select>
      </div>

      <div class="bulk-row">
        <label class="bulk-label" for="bulk-due">Due</label>
        <input id="bulk-due" type="date" class="bulk-input">
      </div>

      <div class="bulk-row">
        <label class="bulk-label" for="bulk-tags-add">Add tags</label>
        <input id="bulk-tags-add" type="text" class="bulk-input" placeholder="comma-separated">
      </div>

      <div class="bulk-row">
        <label class="bulk-label" for="bulk-tags-replace">Replace tags</label>
        <input id="bulk-tags-replace" type="text" class="bulk-input" placeholder="comma-separated (overwrites)">
      </div>

      <div class="bulk-row">
        <label class="bulk-label" for="bulk-blocker-add">Add blocker</label>
        <input id="bulk-blocker-add" type="text" class="bulk-input" placeholder="crumb ID">
      </div>

      <div class="bulk-row">
        <label class="bulk-label" for="bulk-blocker-remove">Remove blocker</label>
        <input id="bulk-blocker-remove" type="text" class="bulk-input" placeholder="crumb ID">
      </div>

      <div class="bulk-actions">
        <button id="bulk-apply-btn" class="btn btn-action">Apply</button>
        <button id="bulk-delete-btn" class="btn btn-danger-solid">Delete all</button>
      </div>
    </div>
  `;

  document.getElementById('bulk-apply-btn').addEventListener('click',  () => handleBulkApply(ids));
  document.getElementById('bulk-delete-btn').addEventListener('click', () => openBulkDeleteModal(ids));
}
```

- [ ] **Step 4: Add bulk-panel CSS to `style.css`**

Append at the end of `crumbs-gui/style.css`:

```css
/* ── Bulk-edit panel ──────────────────────────────────────────────────────── */

#detail-pane.bulk-mode #detail-resizer { display: none; }

.bulk-panel {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px 0;
}

.bulk-header {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-dim);
  margin-bottom: 4px;
}

.bulk-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.bulk-label {
  font-size: 11px;
  color: var(--text-dim);
  width: 88px;
  flex-shrink: 0;
}

.bulk-select,
.bulk-input {
  flex: 1;
  font-size: 12px;
  padding: 3px 6px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--text);
  outline: none;
}

.bulk-select:focus,
.bulk-input:focus {
  border-color: var(--accent);
}

.bulk-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.bulk-actions .btn-danger-solid {
  margin-left: auto;
}
```

- [ ] **Step 5: Verify panel renders**

Run: `just gui-dev`

- Cmd+click two rows → detail pane shows "2 items selected" with all fields, Apply and Delete all buttons.
- Editor column is hidden; properties column expands.
- Escape → panel disappears, normal view restored on next single click.
- Click a single row → normal detail pane reappears.

- [ ] **Step 6: Commit**

```bash
git mit es
git add crumbs-gui/main.js crumbs-gui/style.css
git commit -m "Feat: render bulk-edit panel in detail pane for multi-select"
```

---

## Task 5: `applyBulk()` helper and Apply button wiring

**Files:**
- Modify: `crumbs-gui/main.js`

- [ ] **Step 1: Write `applyBulk(ids, ops)`**

Add this function just above `renderBulkPanel`:

```js
/**
 * Run `ops` (array of `(id) => Promise<void>`) for every ID in `ids`.
 * Errors are collected and displayed after the loop; processing continues
 * regardless of individual failures. Calls `loadItems()` once at the end.
 */
async function applyBulk(ids, ops) {
  const errors = [];
  for (const id of ids) {
    for (const op of ops) {
      try {
        await op(id);
      } catch (e) {
        errors.push(`${id}: ${e}`);
      }
    }
  }
  await loadItems();
  if (errors.length) showError(`Bulk update partially failed:\n${errors.join('\n')}`);
}
```

- [ ] **Step 2: Write `handleBulkApply(ids)`**

Add directly below `applyBulk`:

```js
async function handleBulkApply(ids) {
  const statusEl      = document.getElementById('bulk-status');
  const priorityEl    = document.getElementById('bulk-priority');
  const typeEl        = document.getElementById('bulk-type');
  const dueEl         = document.getElementById('bulk-due');
  const tagsAddEl     = document.getElementById('bulk-tags-add');
  const tagsRepEl     = document.getElementById('bulk-tags-replace');
  const blockerAddEl  = document.getElementById('bulk-blocker-add');
  const blockerRemEl  = document.getElementById('bulk-blocker-remove');

  // Closing has a special modal flow — hand off and return
  if (statusEl.value === 'closed') {
    openBulkCloseModal(ids);
    return;
  }

  const ops = [];

  if (statusEl.value) {
    ops.push(id => invoke('update_status', { dir: storeDir, id, status: statusEl.value }));
  }
  if (priorityEl.value) {
    ops.push(id => invoke('update_priority', { dir: storeDir, id, priority: Number(priorityEl.value) }));
  }
  if (typeEl.value) {
    ops.push(id => invoke('update_type', { dir: storeDir, id, itemType: typeEl.value }));
  }
  if (dueEl.value) {
    ops.push(id => invoke('update_due', { dir: storeDir, id, due: dueEl.value }));
  }
  if (tagsAddEl.value.trim()) {
    const newTags = tagsAddEl.value.split(',').map(t => t.trim()).filter(Boolean);
    ops.push(async id => {
      const item = allItems.find(i => i.id === id);
      const merged = [...new Set([...(item?.tags ?? []), ...newTags])];
      await invoke('update_tags', { dir: storeDir, id, tags: merged.join(',') });
    });
  }
  if (tagsRepEl.value.trim()) {
    const replaceTags = tagsRepEl.value.split(',').map(t => t.trim()).filter(Boolean);
    ops.push(id => invoke('update_tags', { dir: storeDir, id, tags: replaceTags.join(',') }));
  }
  if (blockerAddEl.value.trim()) {
    const blocker = blockerAddEl.value.trim();
    ops.push(id => invoke('link_items', { dir: storeDir, id, relation: 'blocked-by', targets: [blocker], remove: false }));
  }
  if (blockerRemEl.value.trim()) {
    const blocker = blockerRemEl.value.trim();
    ops.push(id => invoke('link_items', { dir: storeDir, id, relation: 'blocked-by', targets: [blocker], remove: true }));
  }

  if (!ops.length) return;

  clearError();
  await applyBulk(ids, ops);
  // Clear the selection after a successful apply
  selectedIds.clear();
  lastClickedId = null;
  updateRowHighlights();
  updateToolbarButtons();
}
```

- [ ] **Step 3: Verify Apply wiring**

Run: `just gui-dev`

- Cmd+click two items. In the panel, change Priority to P1. Click Apply.
- Both items update to P1 in the table. Selection clears.
- Cmd+click two items. Change nothing. Click Apply — nothing happens, no error.
- Cmd+click two items with different types. Type dropdown shows `— mixed —`. Pick "bug". Apply → both become bug.

- [ ] **Step 4: Commit**

```bash
git mit es
git add crumbs-gui/main.js
git commit -m "Feat: implement applyBulk helper and Apply button wiring"
```

---

## Task 6: Bulk close (one reason for all)

**Files:**
- Modify: `crumbs-gui/main.js`

- [ ] **Step 1: Add `pendingBulkCloseIds` state variable**

Near the existing `let pendingCloseId` declaration, add:
```js
let pendingBulkCloseIds = null;   // set when closing multiple items at once
```

- [ ] **Step 2: Write `openBulkCloseModal(ids)`**

Add directly below the existing `openCloseModal` function:

```js
function openBulkCloseModal(ids) {
  pendingBulkCloseIds = ids;
  pendingCloseId      = null;     // ensure single-close path is not triggered
  closeReason.value   = '';
  closeModal.classList.remove('hidden');
  closeReason.focus();
}
```

- [ ] **Step 3: Update `confirmClose()` to handle bulk path**

Find `confirmClose` (around line 1534):
```js
async function confirmClose() {
  closeModal.classList.add('hidden');
  clearError();
  try {
    await invoke('close_item', { dir: storeDir, id: pendingCloseId, reason: closeReason.value.trim() });
    if (primaryId() === pendingCloseId && !showClosedEl.checked) {
      selectedIds.clear(); lastClickedId = null;
    }
    await loadItems();
  } catch (e) {
    showError(`Close failed: ${e}`);
  }
  pendingCloseId = '';
}
```
Replace with:
```js
async function confirmClose() {
  closeModal.classList.add('hidden');
  clearError();
  const reason = closeReason.value.trim();

  if (pendingBulkCloseIds) {
    const ids = pendingBulkCloseIds;
    pendingBulkCloseIds = null;
    await applyBulk(ids, [id => invoke('close_item', { dir: storeDir, id, reason })]);
    selectedIds.clear();
    lastClickedId = null;
    updateRowHighlights();
    updateToolbarButtons();
    return;
  }

  // Single-item close (existing path)
  try {
    await invoke('close_item', { dir: storeDir, id: pendingCloseId, reason });
    if (primaryId() === pendingCloseId && !showClosedEl.checked) {
      selectedIds.clear(); lastClickedId = null;
    }
    await loadItems();
  } catch (e) {
    showError(`Close failed: ${e}`);
  }
  pendingCloseId = '';
}
```

- [ ] **Step 4: Verify bulk close**

Run: `just gui-dev`

- Cmd+click two open items. Change Status to "closed". Click Apply.
- The close modal appears once. Enter a reason. Click confirm.
- Both items close with the same reason. If "show closed" is off, they disappear from the table.
- Selection clears.
- Single-item close via toolbar button still works as before.

- [ ] **Step 5: Commit**

```bash
git mit es
git add crumbs-gui/main.js
git commit -m "Feat: bulk close with single shared reason via existing close modal"
```

---

## Task 7: Bulk delete

**Files:**
- Modify: `crumbs-gui/main.js`

- [ ] **Step 1: Add `pendingBulkDeleteIds` state variable**

Near the existing delete-modal state, add:
```js
let pendingBulkDeleteIds = null;  // set when deleting multiple items at once
```

- [ ] **Step 2: Write `openBulkDeleteModal(ids)`**

Add directly below the existing `openDeleteModal` function:

```js
function openBulkDeleteModal(ids) {
  pendingBulkDeleteIds = ids;
  const msgEl = deleteModal.querySelector('.modal-msg');
  if (msgEl) msgEl.textContent = `Permanently delete ${ids.length} items? This cannot be undone.`;
  deleteModal.classList.remove('hidden');
  deleteConfirmBtn.focus();
}
```

- [ ] **Step 3: Update `confirmDelete()` to handle bulk path**

Find `confirmDelete` (around line 1513):
```js
async function confirmDelete() {
  deleteModal.classList.add('hidden');
  clearError();
  try {
    await invoke('delete_item', { dir: storeDir, id: primaryId() });
    selectedIds.clear(); lastClickedId = null;
    await loadItems();
  } catch (e) {
    showError(`Delete failed: ${e}`);
  }
}
```
Replace with:
```js
async function confirmDelete() {
  deleteModal.classList.add('hidden');
  clearError();

  if (pendingBulkDeleteIds) {
    const ids = pendingBulkDeleteIds;
    pendingBulkDeleteIds = null;
    // Reset modal message for next single-item delete
    const msgEl = deleteModal.querySelector('.modal-msg');
    if (msgEl) msgEl.textContent = 'Permanently delete this item? This cannot be undone.';
    await applyBulk(ids, [id => invoke('delete_item', { dir: storeDir, id })]);
    selectedIds.clear();
    lastClickedId = null;
    updateRowHighlights();
    updateToolbarButtons();
    return;
  }

  // Single-item delete (existing path)
  try {
    await invoke('delete_item', { dir: storeDir, id: primaryId() });
    selectedIds.clear(); lastClickedId = null;
    await loadItems();
  } catch (e) {
    showError(`Delete failed: ${e}`);
  }
}
```

- [ ] **Step 4: Verify bulk delete**

Run: `just gui-dev`

- Cmd+click two items. Click "Delete all" in the bulk panel.
- Modal message reads "Permanently delete 2 items? This cannot be undone."
- Confirm → both items removed from the table. Selection cleared.
- Afterwards, single-item Delete still shows the original single-item message.
- Multi-select + keyboard Delete/Backspace → same bulk modal.

- [ ] **Step 5: Commit**

```bash
git mit es
git add crumbs-gui/main.js
git commit -m "Feat: bulk delete with confirmation modal"
```

---

## Task 8: Mark crumb in-progress, update help modal, smoke-test

**Files:**
- Modify: `crumbs-gui/main.js` (help modal keyboard shortcut table)

- [ ] **Step 1: Mark cr-nk8 in progress**

```bash
/Users/evensolberg/.cargo/bin/crumbs start cr-nk8
```

- [ ] **Step 2: Add multi-select shortcuts to the help modal**

In `crumbs-gui/index.html`, find the "App shortcuts" `<tbody>` (around line 302). Update the existing `Delete / Backspace` and `Escape` rows and insert three new rows so the table reads:

```html
<tr><td><kbd>?</kbd></td><td>Open this help</td></tr>
<tr><td><kbd>Cmd/Ctrl+N</kbd></td><td>New item</td></tr>
<tr><td><kbd>Cmd/Ctrl+F</kbd></td><td>Focus search bar</td></tr>
<tr><td><kbd>Cmd/Ctrl+R</kbd></td><td>Refresh</td></tr>
<tr><td><kbd>Cmd/Ctrl+A</kbd></td><td>Select all filtered rows</td></tr>
<tr><td><kbd>↑ / ↓</kbd></td><td>Navigate rows</td></tr>
<tr><td><kbd>Enter</kbd></td><td>Focus body editor</td></tr>
<tr><td><kbd>Cmd/Ctrl+click</kbd></td><td>Toggle row in/out of multi-selection</td></tr>
<tr><td><kbd>Shift+click</kbd></td><td>Range-select rows</td></tr>
<tr><td><kbd>Delete / Backspace</kbd></td><td>Delete selected item(s)</td></tr>
<tr><td><kbd>Escape</kbd></td><td>Clear multi-selection / dismiss modal</td></tr>
```

- [ ] **Step 3: Full smoke test**

Run: `just gui-dev` and verify the complete flow end-to-end:

1. **Single-select unchanged** — click row, detail pane opens, all fields editable, timer/close/defer toolbar buttons work.
2. **Cmd+click** — toggle two rows, bulk panel appears with correct mixed/shared values.
3. **Shift+click** — range of 4 rows highlights.
4. **Cmd+A** — all filtered rows highlight; filtered to P2 → only P2 rows highlighted.
5. **Escape** — clears multi-selection.
6. **Apply status change** — change priority on 3 items, Apply, all update.
7. **Apply tags merge** — add tag "gui" to 2 items that each have different tags; both gain "gui" while keeping existing tags.
8. **Apply tags replace** — replace all tags on 2 items with "archived"; both tags fields become only "archived".
9. **Apply blocker** — add "cr-abc" as blocker to 2 items.
10. **Bulk close** — status → closed, one reason, both items close.
11. **Bulk delete** — modal shows correct count, confirm removes items.
12. **Keyboard Delete** — multi-select + Backspace opens bulk delete modal.
13. **↑ / ↓ navigation** — works as before on single-select.
14. **No console errors** throughout.

- [ ] **Step 4: Commit**

```bash
git mit es
git add crumbs-gui/main.js crumbs-gui/index.html
git commit -m "Feat: add multi-select shortcuts to help modal; mark cr-nk8 in progress"
```
