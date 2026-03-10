# GUI Editor Enhancements — Design Spec

Date: 2026-03-09
Crumbs: cr-4eq, cr-8r8

## Overview

Two related GUI improvements:
1. **cr-4eq** — After creating a new item, auto-select it and focus the editor.
2. **cr-8r8** — Replace the plain textarea with a CodeMirror 6 editor featuring line
   numbers, markdown syntax highlighting, find/replace, and a collapsible heading
   outline panel.

---

## cr-4eq: Auto-navigate to new crumb

### Problem
After confirming the New Item modal, the item appears in the list but is not
selected. The user must find and click it manually before editing.

### Solution
Modify `confirmNew()` in `main.js`. After `loadItems()` returns, find the newly
created item by title (most-recently-created match, same pattern as
`createNewBlocker()`), set `selectedId`, mark the table row `.selected`, scroll it
into view, call `renderDetail()`, and focus the body editor.

### Scope
- One function modified: `confirmNew()` (~8 lines added).
- No new Tauri commands required.

---

## cr-8r8: CodeMirror 6 editor + heading outline

### Approach
Replace `<textarea id="detail-text">` with a CodeMirror 6 `EditorView` mounted on
`<div id="detail-editor">`. Bundle CM6 as a local ESM file (no CDN, no ongoing
build step) to match the existing pattern of local assets (`marked.min.js`).

### Bundle
- Entry file: `crumbs-gui/codemirror-entry.js` (committed for reproducibility)
- Output: `crumbs-gui/codemirror.bundle.js` (committed built artifact)
- Build command: `npx esbuild --bundle --format=esm --outfile=crumbs-gui/codemirror.bundle.js crumbs-gui/codemirror-entry.js`
- Approximate size: 200–250 KB minified

### CM6 extensions included

| Extension | Purpose |
|-----------|---------|
| `lineNumbers` | Line numbers in gutter |
| `highlightActiveLine`, `highlightActiveLineGutter` | Highlight current line |
| `markdown` + `syntaxHighlighting` + `defaultHighlightStyle` | Markdown syntax colours |
| `history` + `historyKeymap` | Undo/redo |
| `defaultKeymap` + `indentWithTab` | Standard keyboard behaviour |
| `closeBrackets` + `closeBracketsKeymap` | Auto-close `[`, `(`, `` ` ``, `*` |
| `highlightSelectionMatches` | Highlight all occurrences of selection |
| `search` + `searchKeymap` | Cmd/Ctrl-F find/replace panel |
| `placeholder` | Native CM6 placeholder text |
| `drawSelection`, `dropCursor`, `highlightSpecialChars` | Polish |

### Theming
`EditorView.theme()` uses the app's existing CSS variables (`--bg`, `--text`,
`--accent`, `--border`, `--bg-alt`, etc.) so light/dark toggling works
automatically with no extra code. The CM6 search panel may need a few extra CSS
rules to match the app's visual style.

### Integration points (textarea → CM6)

| Old (`detailText`) | New (CM6 `view`) |
|--------------------|-----------------|
| `detailText.value = body` | `view.dispatch({ changes: { from:0, to: doc.length, insert: body } })` |
| `detailText.value` (read) | `view.state.doc.toString()` |
| `addEventListener('input', …)` | `updateListener` extension |
| `addEventListener('blur', …)` | `domEventHandlers({ blur })` |
| Cmd+S keydown handler | CM6 keymap entry |
| `insertAtCursor(detailText, char)` | `view.dispatch` at cursor head |
| `classList.toggle('hidden', on)` | Toggle visibility of editor container div |

### Heading outline panel

**Layout:** `#detail-right` contains a new `#editor-area` flex-row wrapper holding:
- `#detail-editor` (CM6, `flex: 1`)
- `#outline-panel` (fixed ~180px width, collapsible, `border-left`)

A toggle button (`≡`) in the `panel-title` bar shows/hides the outline. State is
persisted to `localStorage`.

**Content:** On CM6 `updateListener` changes (debounced ~300ms), headings are
parsed from the document with `/^#{1,6} .+/gm`. Each heading renders as a list
item indented by `(level - 1) * 0.75rem`. The outline is only updated while
visible.

**Known limitation:** Headings inside fenced code blocks will appear in the
outline. Filtering them correctly requires tracking fence state; this is deferred
as a low-priority follow-up.

**Click to navigate:**
```js
const line = view.state.doc.line(headingLineNumber);
view.dispatch({ selection: { anchor: line.from }, scrollIntoView: true });
view.focus();
```
Works in edit mode only. Outline is hidden when preview mode is active.

### Known drawbacks
- **Code-block headings in outline** — see Known limitation above.
- **Narrow pane cramping** — outline can crowd the editor at small widths; the
  toggle mitigates this. No auto-hide at narrow sizes in v1.
- **CM6 search panel styling** — may need manual CSS tweaks to match app theme.
- **Tab behaviour change** — Tab now indents (via `indentWithTab`) rather than
  moving focus. Correct for an editor; minor UX change from the old textarea.
- **Bundle size** — ~250 KB committed blob. Irrelevant for a local Tauri app.

### Files changed

| File | Change |
|------|--------|
| `crumbs-gui/codemirror-entry.js` | New — CM6 bundle entry point |
| `crumbs-gui/codemirror.bundle.js` | New — built CM6 ESM bundle |
| `crumbs-gui/index.html` | Replace textarea with editor div; add outline panel markup and toggle button |
| `crumbs-gui/main.js` | CM6 init; replace all textarea refs; outline parser + toggle; auto-navigate after create |
| `crumbs-gui/style.css` | Editor wrapper, outline panel, CM6 gutter/search panel overrides |
