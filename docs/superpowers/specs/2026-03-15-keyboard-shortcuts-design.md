# Keyboard Shortcuts & Help Modal — Design Spec

**Date:** 2026-03-15
**Status:** Approved

## Overview

Add a comprehensive keyboard shortcut system to crumbs-gui, plus a help modal reachable via `?` key or toolbar button. Also restructures the left side of the toolbar.

## Scope

- Global app-level shortcuts (document `keydown`)
- CM6 editor addition: `Cmd+K` link snippet
- Help modal with shortcuts, status symbols, and ID tips
- Toolbar restructure: move `?` and theme buttons to left cluster

---

## 1. Toolbar Restructure

**Current left cluster:** `☰ sidebar-toggle | sep | + New | …`
**New left cluster:** `? help | ☰ sidebar-toggle | ☀ theme | sep | + New | …`

- The `?` button is added as the leftmost element, before the sidebar toggle.
- The `☀ theme` button moves from `toolbar-right-controls` to immediately right of the sidebar toggle.
- `toolbar-right-controls` after the change contains: `show-closed` toggle label, `col-picker`, `reindex`, `refresh`. The theme button is removed from this group.
- The `themeBtn` JS reference uses `document.getElementById('theme-btn')` and does not need to change — the button ID stays the same.

No layout changes to toolbar row 2 or the detail pane.

---

## 2. Help Modal

A new `#help-modal` added to `index.html`. Must use `class="modal hidden"` so `isModalOpen()` detects it correctly. Three sections, no action buttons — dismissed by `Escape` or click-outside only (intentional; this is a read-only reference panel).

### 2a. Keyboard Shortcuts

Displayed as two groups: **App** and **Editor (when body editor is focused)**.

**App**

| Shortcut | Action |
|---|---|
| `?` | Open this help |
| `Cmd/Ctrl+N` | New item |
| `Cmd/Ctrl+F` | Focus search bar |
| `Cmd/Ctrl+R` | Refresh |
| `↑` / `↓` | Navigate rows |
| `Enter` | Focus body editor |
| `Delete` / `Backspace` | Delete selected item |
| `Escape` | Dismiss modal / close context menu |

**Editor**

| Shortcut | Action |
|---|---|
| `Cmd/Ctrl+S` | Save |
| `Cmd/Ctrl+B` | Bold |
| `Cmd/Ctrl+I` | Italic |
| `Cmd/Ctrl+K` | Insert link |
| `Cmd/Ctrl+F` | Find / Replace |
| `Cmd/Ctrl+D` | Delete line |
| `Cmd/Ctrl+↑/↓` | Move line up / down |
| `Cmd/Ctrl+0–6` | Heading level |

The two-group layout removes ambiguity: `Cmd+F` appears once in each group with distinct meanings.

### 2b. Status Symbols

| Symbol | Meaning |
|---|---|
| ○ | Open |
| ● | In Progress |
| ⊘ | Blocked |
| ◷ | Deferred |
| ✓ | Closed |

### 2c. ID Tips

- Bare suffix works: `x7q` resolves to `cr-x7q`
- IDs are case-insensitive: `CR-X7Q` = `cr-x7q`

**Dismiss:** `Escape` key (handled in the document-level `keydown` handler, consistent with all other modals — not on the modal element itself) or click outside the modal box. No close button; the modal is intentionally action-free.

---

## 3. Global Keyboard Shortcuts

All handled in `document.addEventListener('keydown', …)`.

### 3a. Focus Guard

A shared helper suppresses navigation/destructive shortcuts when a text field or CM6 editor is active. The inline title editor in the detail pane uses an `<input>` element (not `contenteditable`), so the tag checks below are sufficient:

```js
function isInputFocused() {
  const el = document.activeElement;
  return el && (
    el.tagName === 'INPUT' ||
    el.tagName === 'TEXTAREA' ||
    el.tagName === 'SELECT' ||
    !!el.closest('.cm-editor')
  );
}
```

### 3b. Modal Guard

A helper to check whether any modal is currently visible:

```js
function isModalOpen() {
  return !!document.querySelector('.modal:not(.hidden)');
}
```

> **Convention:** All modals must have the `modal` CSS class. Any future modal that omits it will be invisible to this guard. The new `#help-modal` must use `class="modal hidden"`.

### 3c. Shortcut Table

| Key | Condition | Action |
|---|---|---|
| `?` (`e.key === '?'`) | `!isInputFocused() && !isModalOpen()` | Show `#help-modal` |
| `Cmd+N` | `!isInputFocused() && !isModalOpen()` | Call `openNewModal()` |
| `Cmd+F` | `!isInputFocused()` | Focus `#search-input`, `select()` |
| `Cmd+R` | always | `preventDefault()`, call `loadItems()` |
| `↑` | `!isInputFocused()` | Select previous table row |
| `↓` | `!isInputFocused()` | Select next table row |
| `Enter` | `!isInputFocused() && selectedId && detail pane visible` | Focus CM6 editor |
| `Delete`/`Backspace` | `!isInputFocused() && selectedId` | Open delete modal |
| `Escape` | help modal visible | Hide `#help-modal` |

**Notes:**

- `?` uses `e.key === '?'` (not `e.code`). On macOS, `?` is always `Shift+/`; no `AltGr` variant exists, so no additional modifier check is needed.
- `Cmd+F` when a non-CM6 input (e.g., tag filter, title input) is focused: `isInputFocused()` returns `true` and the shortcut is suppressed with no action. This is intentional — those inputs handle their own text editing.
- `Cmd+R`: `loadItems()` already guards against a null/empty `storeDir`. In Tauri 2, the webview does not expose a native reload menu item by default, so `preventDefault()` on the keydown event is sufficient. Verify this holds if `tauri.conf.json` capabilities are expanded in future.
- `Cmd+R` calls `loadItems()` which guards `storeDir` — no additional null check needed.
- Row navigation (`↑`/`↓`) clamps at first/last row (no wrap). Order of operations: (1) update `selectedId`, (2) update `.selected` CSS class on the table row, (3) call `renderDetail(selectedItem())`, (4) call `scrollIntoView({ block: 'nearest' })` on the new row. This matches the existing row-click handler.
- `Enter` to focus editor: call `view.focus()`. Only fires when `#detail-pane` does not have the `hidden` class.

---

## 4. CM6 Editor: `Cmd+K` Link Snippet

### 4a. codemirror-entry.js

Add `snippet` to the existing `@codemirror/autocomplete` import (do not create a second import for the same module):

```js
// Before
import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';

// After
import { closeBrackets, closeBracketsKeymap, snippet } from '@codemirror/autocomplete';
```

Export `snippet` alongside the other named exports already re-exported from this file.

After editing `codemirror-entry.js`, rebuild the bundle before testing:

```sh
npm run build-cm6
```

### 4b. main.js — keymap entry

The existing custom bindings (`Mod-s`, `Mod-b`, `Mod-i`, etc.) are listed after the spread keymaps. `Mod-k` is unbound in all four spread keymaps, so the same placement is safe.

Add `Mod-k` alongside the existing custom entries, after the spreads:

```js
{ key: 'Mod-k', run: snippet('[${text}](${url})') },
```

Tab stops: cursor lands on `text` (selected) → `Tab` → `url` → `Tab`/`Escape` exits snippet mode.

> **Note:** `closeBracketsKeymap` may auto-pair `]` and `)` during snippet insertion. Verify during testing that the result is `[text](url)` and not `[text]](url))`. If auto-pairing fires, fall back to a plain `insertText('[]()')` with manual cursor placement inside `[]`.

### 4c. Pre-existing `Mod-d` conflict

`searchKeymap` binds `Mod-d` to `selectNextOccurrence`, and it is spread before the custom entries, so it takes precedence. The existing `Mod-d → deleteLine` custom entry is currently unreachable. This is a pre-existing bug outside the scope of this spec. To fix it, move the `Mod-d` custom entry to before `...searchKeymap` in the keymap array. This fix may be done alongside this work or deferred.

---

## 5. Files Changed

| File | Change |
|---|---|
| `crumbs-gui/index.html` | Add `#help-modal` (with `class="modal hidden"`); add `?` button; move `☀` button to left cluster |
| `crumbs-gui/main.js` | Add `isInputFocused`, `isModalOpen` helpers; extend global `keydown`; wire help modal open/close; wire `?` button click |
| `crumbs-gui/codemirror-entry.js` | Add `snippet` to existing `@codemirror/autocomplete` import; re-export it |
| `crumbs-gui/style.css` | Minor additions for two-group shortcut table layout in `#help-modal` (existing modal styles handle the rest) |

---

## 6. Out of Scope

- CLI shortcuts (no applicable analog)
- Customizable keybindings
- Shortcut hints in button tooltips (potential follow-up)
- Fix for pre-existing `Mod-d` / `searchKeymap` conflict (noted in §4c)
