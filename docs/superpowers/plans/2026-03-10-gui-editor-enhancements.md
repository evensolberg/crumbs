# GUI Editor Enhancements Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the plain textarea in crumbs-gui with a CodeMirror 6 editor (line numbers, markdown syntax highlighting, find/replace, collapsible heading outline), and auto-select newly created items.

**Architecture:** CM6 is bundled once as a local ESM file (`crumbs-gui/dist/codemirror.bundle.js`) using esbuild and committed — same pattern as `marked.min.js`. The `<textarea id="detail-text">` is replaced by `<div id="detail-editor">` with a CM6 `EditorView` mounted on it. All `detailText.*` call sites (9 references across 8 locations) are updated to use the CM6 API. The heading outline is a fixed-width collapsible `<div>` alongside the editor in a new `#editor-area` flex-row wrapper, populated by a regex heading parser on every editor change. Auto-navigate wires into `confirmNew()` after `loadItems()` returns.

**Tech Stack:** CodeMirror 6 (`@codemirror/*`), esbuild (one-time bundle build), Tauri 2, vanilla JS.

**Spec:** `docs/superpowers/specs/2026-03-09-gui-editor-enhancements-design.md`

**Crumbs:** cr-4eq (auto-navigate), cr-8r8 (CM6 editor + outline)

---

## File map

| File | Change |
|------|--------|
| `crumbs-gui/codemirror-entry.js` | **New** — CM6 bundle entry (source, committed for reproducibility) |
| `crumbs-gui/package.json` | **New** — npm dev deps + `build-cm6` script |
| `crumbs-gui/package-lock.json` | **New** — lockfile (committed) |
| `crumbs-gui/dist/codemirror.bundle.js` | **New** — built CM6 ESM bundle (committed like `marked.min.js`) |
| `crumbs-gui/index.html` | Replace textarea; add `#editor-area`, `#detail-editor`, `#outline-panel`, outline toggle button |
| `crumbs-gui/main.js` | CM6 init; replace all `detailText.*`; outline parser + toggle; auto-navigate after create |
| `crumbs-gui/style.css` | `#editor-area`, `#detail-editor`, `#outline-panel`, `.outline-item`, CM6 gutter + search panel overrides |
| `.gitignore` | Add `/crumbs-gui/node_modules` |

**Key constraint:** `just gui-dev` copies `crumbs-gui/{index.html,main.js,style.css}` to `crumbs-gui/dist/` on every run. The bundle lives directly in `crumbs-gui/dist/` (committed) so no justfile changes are needed. Tauri's `frontendDist` is `../dist`.

**Import resolution:** `main.js` is loaded from `crumbs-gui/dist/`, so `import ... from './codemirror.bundle.js'` resolves to `crumbs-gui/dist/codemirror.bundle.js`. The `package.json` build script outputs there directly (`--outfile=dist/codemirror.bundle.js` run from `crumbs-gui/`). Do not put the bundle in `crumbs-gui/` root — it won't be found at runtime.

---

## Chunk 1: CM6 bundle

### Task 1: Create entry file, package.json, and build the bundle

**Files:**
- Create: `crumbs-gui/codemirror-entry.js`
- Create: `crumbs-gui/package.json`
- Create: `crumbs-gui/dist/codemirror.bundle.js` (generated)
- Modify: `.gitignore`

- [ ] **Step 1: Add `crumbs-gui/node_modules` to `.gitignore`**

  Edit `.gitignore` (project root), add:
  ```
  /crumbs-gui/node_modules
  ```

- [ ] **Step 2: Create `crumbs-gui/package.json`**

  ```json
  {
    "name": "crumbs-gui-bundle",
    "private": true,
    "scripts": {
      "build-cm6": "esbuild codemirror-entry.js --bundle --format=esm --minify --outfile=dist/codemirror.bundle.js"
    },
    "devDependencies": {
      "@codemirror/autocomplete": "^6",
      "@codemirror/commands": "^6",
      "@codemirror/lang-markdown": "^6",
      "@codemirror/language": "^6",
      "@codemirror/search": "^6",
      "@codemirror/state": "^6",
      "@codemirror/view": "^6",
      "esbuild": "^0.25"
    }
  }
  ```

- [ ] **Step 3: Create `crumbs-gui/codemirror-entry.js`**

  ```js
  // CodeMirror 6 bundle entry — exports everything used by crumbs-gui/main.js.
  // To rebuild: cd crumbs-gui && npm run build-cm6

  export {
    EditorView,
    keymap,
    lineNumbers,
    highlightActiveLine,
    highlightActiveLineGutter,
    drawSelection,
    dropCursor,
    highlightSpecialChars,
    placeholder,
  } from '@codemirror/view';

  export { EditorState } from '@codemirror/state';

  export {
    defaultKeymap,
    history,
    historyKeymap,
    indentWithTab,
  } from '@codemirror/commands';

  export {
    closeBrackets,
    closeBracketsKeymap,
  } from '@codemirror/autocomplete';

  export {
    search,
    searchKeymap,
    highlightSelectionMatches,
  } from '@codemirror/search';

  export {
    syntaxHighlighting,
    defaultHighlightStyle,
  } from '@codemirror/language';

  export { markdown } from '@codemirror/lang-markdown';
  ```

- [ ] **Step 4: Install deps and build the bundle**

  ```bash
  cd crumbs-gui
  npm install
  npm run build-cm6
  ```

  Expected: `crumbs-gui/dist/codemirror.bundle.js` created, ~200–250 KB.

- [ ] **Step 5: Verify the bundle is valid ESM**

  ```bash
  head -c 100 crumbs-gui/dist/codemirror.bundle.js
  ```

  Expected: starts with minified JS (e.g. `var ...` or `(()=>{...`), not an error message.

- [ ] **Step 6: Commit**

  ```bash
  git add crumbs-gui/codemirror-entry.js crumbs-gui/package.json crumbs-gui/package-lock.json crumbs-gui/dist/codemirror.bundle.js .gitignore
  git mit es
  git commit -m "Chore: add CodeMirror 6 bundle for GUI editor upgrade"
  ```

---

## Chunk 2: Replace textarea with CM6

### Task 2: Update `index.html`

**Files:**
- Modify: `crumbs-gui/index.html` (lines 129–139, the `#detail-right` section)

- [ ] **Step 1: Replace the `#detail-right` inner content**

  Find the current `#detail-right` div:
  ```html
  <div id="detail-right">
    <div class="panel-title">
      <span id="detail-title-label"></span>
      <div class="panel-title-actions">
        <button type="button" id="emoji-btn" class="btn btn-secondary btn-small" title="Insert emoji" disabled>😀</button>
        <button type="button" id="preview-btn" class="btn btn-secondary btn-small" title="Toggle markdown preview">Preview</button>
      </div>
    </div>
    <textarea id="detail-text" placeholder="No body text."></textarea>
    <div id="detail-preview" class="hidden markdown-body"></div>
  </div>
  ```

  Replace with:
  ```html
  <div id="detail-right">
    <div class="panel-title">
      <span id="detail-title-label"></span>
      <div class="panel-title-actions">
        <button type="button" id="outline-toggle-btn" class="btn btn-secondary btn-small" title="Toggle heading outline">&#8801;</button>
        <button type="button" id="emoji-btn" class="btn btn-secondary btn-small" title="Insert emoji" disabled>😀</button>
        <button type="button" id="preview-btn" class="btn btn-secondary btn-small" title="Toggle markdown preview">Preview</button>
      </div>
    </div>
    <div id="editor-area">
      <div id="detail-editor"></div>
      <div id="outline-panel" class="hidden">
        <div id="outline-list"></div>
      </div>
    </div>
    <div id="detail-preview" class="hidden markdown-body"></div>
  </div>
  ```

- [ ] **Step 2: Verify HTML parses cleanly**

  Run `just gui-dev`. Check the console for parse errors only. **Do not select any item** — at this point `main.js` still references `detailText` (which no longer exists), so selecting an item will throw a TypeError. That is expected and will be fixed in Task 3–4.

---

### Task 3: Initialize CM6 in `main.js`

**Files:**
- Modify: `crumbs-gui/main.js`

- [ ] **Step 1: Add CM6 imports at the top of `main.js`**

  Insert before any existing code at line 1:
  ```js
  import {
    EditorView, keymap, lineNumbers, highlightActiveLine,
    highlightActiveLineGutter, drawSelection, dropCursor,
    highlightSpecialChars, placeholder,
  } from './codemirror.bundle.js';
  import { EditorState } from './codemirror.bundle.js';
  import { defaultKeymap, history, historyKeymap, indentWithTab } from './codemirror.bundle.js';
  import { closeBrackets, closeBracketsKeymap } from './codemirror.bundle.js';
  import { search, searchKeymap, highlightSelectionMatches } from './codemirror.bundle.js';
  import { syntaxHighlighting, defaultHighlightStyle } from './codemirror.bundle.js';
  import { markdown } from './codemirror.bundle.js';
  ```

- [ ] **Step 2: Remove the `detailText` element reference**

  Find and delete line 75:
  ```js
  const detailText       = document.getElementById('detail-text');
  ```

- [ ] **Step 3: Add new element references**

  In the same block of `const` element references (around line 75), add:
  ```js
  const detailEditorEl   = document.getElementById('detail-editor');
  const outlinePanel     = document.getElementById('outline-panel');
  const outlineList      = document.getElementById('outline-list');
  const outlineToggleBtn = document.getElementById('outline-toggle-btn');
  ```

- [ ] **Step 4: Define the CM6 theme**

  After the element references block, add:
  ```js
  // ── CodeMirror theme (inherits app CSS variables) ─────────────────────────
  const appTheme = EditorView.theme({
    '&': {
      height: '100%',
      background: 'var(--bg)',
      color: 'var(--text)',
      fontSize: '13px',
      fontFamily: 'inherit',
    },
    '.cm-scroller': { overflow: 'auto' },
    '.cm-content': { caretColor: 'var(--accent)', padding: '8px 0' },
    '.cm-cursor': { borderLeftColor: 'var(--accent)' },
    '.cm-gutters': {
      background: 'var(--bg-alt, var(--bg))',
      color: 'var(--text-muted, #888)',
      border: 'none',
      borderRight: '1px solid var(--border)',
    },
    '.cm-activeLineGutter': { background: 'var(--bg-hover, rgba(0,0,0,.05))' },
    '.cm-activeLine':        { background: 'var(--bg-hover, rgba(0,0,0,.05))' },
    '.cm-selectionBackground, ::selection': {
      background: 'var(--accent-muted, rgba(0,120,255,.2))',
    },
    '.cm-searchMatch': {
      background: 'var(--accent-muted, rgba(0,120,255,.2))',
      outline: '1px solid var(--accent)',
    },
    '.cm-searchMatch.cm-searchMatch-selected': {
      background: 'var(--accent)',
      color: 'var(--bg)',
    },
    '.cm-panels': {
      background: 'var(--bg-alt, var(--bg))',
      color: 'var(--text)',
      borderTop: '1px solid var(--border)',
    },
    '.cm-panels input, .cm-panels button': {
      background: 'var(--bg)',
      color: 'var(--text)',
      border: '1px solid var(--border)',
      borderRadius: '3px',
    },
  });
  ```

  Note: CSS variables mean this adapts to the existing light/dark theme toggle automatically.

- [ ] **Step 5: Initialize the `EditorView`**

  After the theme definition:
  ```js
  // ── CodeMirror editor instance ─────────────────────────────────────────────
  let view = new EditorView({
    state: EditorState.create({
      doc: '',
      extensions: [
        appTheme,
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        highlightSpecialChars(),
        drawSelection(),
        dropCursor(),
        history(),
        EditorState.allowMultipleSelections.of(true),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        markdown(),
        closeBrackets(),
        search({ top: false }),
        highlightSelectionMatches(),
        placeholder('No body text.'),
        keymap.of([
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...historyKeymap,
          ...searchKeymap,
          indentWithTab,
          { key: 'Mod-s', run: () => { flushAutosave(); return true; } },
        ]),
        EditorView.updateListener.of(update => {
          if (update.docChanged) {
            scheduleAutosave();
            scheduleOutlineUpdate(); // defined in Task 6; hoisted as a function declaration
          }
        }),
        EditorView.domEventHandlers({
          blur: () => { flushAutosave(); },
        }),
      ],
    }),
    parent: detailEditorEl,
  });
  ```

  Note: `flushAutosave` and `scheduleAutosave` are referenced here before their definitions — this is fine in JS (they are function declarations hoisted, or will be defined in the same module scope). If they are `function` declarations, no change needed. If they are `const` arrow functions, move the `view` init below them.

- [ ] **Step 6: Verify the editor renders**

  Run `just gui-dev`, select any item. The CM6 editor should appear in the detail pane. Console should be clean. Body text will be blank (wired in Task 4).

---

### Task 4: Update all `detailText` call sites

**Files:**
- Modify: `crumbs-gui/main.js`

There are 9 references across 8 locations. Update them in order.

- [ ] **Step 1: Declare `outlineVisible` and update `setPreviewMode` (~line 629)**

  First, add `outlineVisible` near the other state variables at the top of `main.js` (around line 7, near `let selectedId = null`):
  ```js
  let outlineVisible = localStorage.getItem('outlineVisible') === 'true';
  ```

  Then update `setPreviewMode`:

  Find:
  ```js
  function setPreviewMode(on) {
    previewMode = on;
    previewBtn.textContent = on ? 'Edit' : 'Preview';
    detailText.classList.toggle('hidden', on);
    detailPreview.classList.toggle('hidden', !on);
    if (on) {
      detailPreview.innerHTML = marked.parse(expandEmoji(detailText.value || ''));
    }
  }
  ```

  Replace with:
  ```js
  function setPreviewMode(on) {
    previewMode = on;
    previewBtn.textContent = on ? 'Edit' : 'Preview';
    detailEditorEl.classList.toggle('hidden', on);
    outlinePanel.classList.toggle('hidden', on || !outlineVisible);
    detailPreview.classList.toggle('hidden', !on);
    if (on) {
      detailPreview.innerHTML = marked.parse(expandEmoji(view.state.doc.toString()));
    }
  }
  ```


- [ ] **Step 2: Update `renderDetail` (~line 654)**

  Find:
  ```js
    loadedBody = item.description ?? '';
    detailText.value = loadedBody;
    setPreviewMode(false);
  ```

  Replace with:
  ```js
    loadedBody = item.description ?? '';
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: loadedBody },
    });
    setPreviewMode(false);
  ```

- [ ] **Step 3: Update `scheduleAutosave` and `flushAutosave` (~line 1540)**

  Find:
  ```js
  function scheduleAutosave() {
    if (!selectedId) return;
    clearTimeout(autosaveTimer);
    autosaveTimer = setTimeout(() => {
      if (detailText.value !== loadedBody) doSaveText(selectedId, detailText.value);
    }, 2000);
  }
  function flushAutosave() {
    if (!selectedId) return;
    clearTimeout(autosaveTimer);
    autosaveTimer = null;
    if (detailText.value !== loadedBody) doSaveText(selectedId, detailText.value);
  }
  ```

  Replace with:
  ```js
  function scheduleAutosave() {
    if (!selectedId) return;
    clearTimeout(autosaveTimer);
    autosaveTimer = setTimeout(() => {
      const text = view.state.doc.toString();
      if (text !== loadedBody) doSaveText(selectedId, text);
    }, 2000);
  }
  function flushAutosave() {
    if (!selectedId) return;
    clearTimeout(autosaveTimer);
    autosaveTimer = null;
    const text = view.state.doc.toString();
    if (text !== loadedBody) doSaveText(selectedId, text);
  }
  ```

- [ ] **Step 4: Remove the three old `detailText` event listeners (~line 1553)**

  Find and delete:
  ```js
  detailText.addEventListener('input', scheduleAutosave);
  detailText.addEventListener('blur', flushAutosave);
  detailText.addEventListener('keydown', e => {
    if (e.key === 's' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      flushAutosave();
    }
  });
  ```

  These are now handled by the CM6 `updateListener`, `domEventHandlers`, and keymap in the `EditorView` setup.

- [ ] **Step 5: Update `insertAtCursor` for emoji (~line 1753)**

  Find:
  ```js
  function insertAtCursor(el, text) {
    const s = el.selectionStart, e = el.selectionEnd;
    el.value = el.value.slice(0, s) + text + el.value.slice(e);
    el.selectionStart = el.selectionEnd = s + text.length;
    el.dispatchEvent(new Event('input'));
    el.focus();
  }
  ```

  Replace with:
  ```js
  function insertAtCursor(_el, text) {
    view.dispatch(view.state.replaceSelection(text));
    view.focus();
  }
  ```

  The `_el` parameter is kept so the call site at line 1785 (`insertAtCursor(detailText, char)`) still parses. Change `detailText` to `null` at that call site too, since `_el` is now ignored:
  ```js
  insertAtCursor(null, char);
  ```

- [ ] **Step 6: Run `just gui-dev` and do a full manual test**

  Verify each:
  - Select an item → body loads in CM6 editor
  - Edit body → autosaves after 2 s (check with `crumbs show <id>` in terminal)
  - Cmd+S → saves immediately
  - Click elsewhere (blur) → saves immediately
  - Preview → renders markdown correctly
  - Edit → editor returns with content intact
  - Emoji button → emoji inserts at cursor position
  - Undo (Cmd+Z) → works
  - Cmd+F → CM6 search panel opens
  - Line numbers visible in gutter

- [ ] **Step 7: Commit**

  ```bash
  git add crumbs-gui/index.html crumbs-gui/main.js
  git mit es
  git commit -m "Feat: replace textarea with CodeMirror 6 editor in GUI"
  ```

---

## Chunk 3: Styles, outline panel, auto-navigate

### Task 5: Update `style.css` for the editor area and outline panel

**Files:**
- Modify: `crumbs-gui/style.css`

- [ ] **Step 1: Update `#detail-right` and add `#editor-area` styles**

  Find the existing `#detail-right` rule in `style.css`. Ensure it is a flex column:
  ```css
  #detail-right {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }
  ```

  Add new rules:
  ```css
  #editor-area {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  #detail-editor {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  /* CM6 internals must fill the container */
  #detail-editor .cm-editor {
    height: 100%;
  }

  #detail-editor .cm-scroller {
    overflow: auto;
  }

  #outline-panel {
    width: 180px;
    min-width: 180px;
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    overflow-y: auto;
    padding: 6px 0;
    font-size: 12px;
  }

  #outline-panel.hidden {
    display: none;
  }

  .outline-item {
    display: block;
    padding: 3px 8px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-muted, #888);
    border-radius: 3px;
    line-height: 1.6;
  }

  .outline-item:hover {
    background: var(--bg-hover, rgba(0,0,0,.05));
    color: var(--text);
  }

  .outline-empty {
    padding: 8px;
    color: var(--text-muted, #888);
    font-style: italic;
  }
  ```

- [ ] **Step 2: Verify layout in `just gui-dev`**

  Select an item. Confirm:
  - CM6 editor fills the available width
  - Outline panel is hidden (correct — it starts hidden)
  - No layout regressions in other parts of the UI

---

### Task 6: Implement the heading outline panel

**Files:**
- Modify: `crumbs-gui/main.js`

- [ ] **Step 1: Confirm `outlineVisible` is declared**

  `outlineVisible` was declared in Task 4 Step 1. Verify it is present near the top of `main.js`:
  ```js
  let outlineVisible = localStorage.getItem('outlineVisible') === 'true';
  ```

- [ ] **Step 2: Apply initial outline visibility on load**

  After the `outlineToggleBtn` element reference, add:
  ```js
  outlinePanel.classList.toggle('hidden', !outlineVisible);
  outlineToggleBtn.classList.toggle('active', outlineVisible);
  ```

- [ ] **Step 3: Wire the toggle button**

  Add:
  ```js
  outlineToggleBtn.addEventListener('click', () => {
    outlineVisible = !outlineVisible;
    localStorage.setItem('outlineVisible', outlineVisible);
    outlinePanel.classList.toggle('hidden', !outlineVisible || previewMode);
    outlineToggleBtn.classList.toggle('active', outlineVisible);
    if (outlineVisible) renderOutline();
  });
  ```

- [ ] **Step 4: Add `renderOutline` and `scheduleOutlineUpdate`**

  Add after `setPreviewMode`:
  ```js
  let outlineDebounceTimer = null;

  function renderOutline() {
    if (!outlineVisible || previewMode) return;
    const doc = view.state.doc;
    const headingRe = /^(#{1,6}) (.+)/;
    const items = [];
    for (let i = 1; i <= doc.lines; i++) {
      const line = doc.line(i);
      const m = line.text.match(headingRe);
      if (m) items.push({ level: m[1].length, text: m[2], lineNum: i });
    }
    if (items.length === 0) {
      outlineList.innerHTML = '<div class="outline-empty">No headings</div>';
      return;
    }
    outlineList.innerHTML = '';
    for (const { level, text, lineNum } of items) {
      const el = document.createElement('div');
      el.className = 'outline-item';
      el.style.paddingLeft = `${6 + (level - 1) * 10}px`;
      el.title = text;
      el.textContent = text;
      el.addEventListener('click', () => {
        const target = view.state.doc.line(lineNum);
        view.dispatch({ selection: { anchor: target.from }, scrollIntoView: true });
        view.focus();
      });
      outlineList.appendChild(el);
    }
  }

  function scheduleOutlineUpdate() {
    if (!outlineVisible || previewMode) return;
    clearTimeout(outlineDebounceTimer);
    outlineDebounceTimer = setTimeout(renderOutline, 300);
  }
  ```

- [ ] **Step 5: Verify `updateListener` already calls `scheduleOutlineUpdate`**

  The `EditorView.updateListener.of(...)` written in Task 3 Step 5 already includes `scheduleOutlineUpdate()`. Confirm it reads:
  ```js
  EditorView.updateListener.of(update => {
    if (update.docChanged) {
      scheduleAutosave();
      scheduleOutlineUpdate();
    }
  }),
  ```
  If it is missing the `scheduleOutlineUpdate()` call, add it now. Do **not** add a second `updateListener` — there must be exactly one.

- [ ] **Step 6: Call `renderOutline` when an item is selected**

  In `renderDetail`, after `setPreviewMode(false)`:
  ```js
    renderOutline();
  ```

- [ ] **Step 7: Manual test**

  Create or find an item with markdown headings (`# Foo`, `## Bar`, `### Baz`) in its body.
  - Click `≡` toggle → outline panel appears with headings indented by level
  - Click a heading → editor scrolls to that line, cursor placed there
  - Click `≡` again → panel hides
  - Close and reopen app → outline visibility remembered
  - Switch to Preview → outline hides; switch back to Edit → outline returns
  - Item with no headings → shows "No headings"

- [ ] **Step 8: Commit**

  ```bash
  git add crumbs-gui/main.js crumbs-gui/style.css
  git mit es
  git commit -m "Feat: add collapsible heading outline panel to GUI editor (cr-8r8)"
  ```

---

### Task 7: Auto-navigate to newly created item (cr-4eq)

**Files:**
- Modify: `crumbs-gui/main.js`

- [ ] **Step 1: Update `confirmNew` (~line 1084)**

  Find:
  ```js
  async function confirmNew() {
    const title = newTitle.value.trim();
    if (!title) return;
    newModal.classList.add('hidden');
    clearError();
    try {
      await invoke('create_item', { dir: storeDir, title });
      await loadItems();
    } catch (e) {
      showError(`Create failed: ${e}`);
    }
  }
  ```

  Replace with:
  ```js
  async function confirmNew() {
    const title = newTitle.value.trim();
    if (!title) return;
    newModal.classList.add('hidden');
    clearError();
    try {
      await invoke('create_item', { dir: storeDir, title });
      await loadItems();
      // Newly created items are always `open`, so they appear in allItems regardless
      // of the showClosed toggle. Matching by title is safe: sort picks the most
      // recently created item if there are duplicates.
      const newItem = allItems
        .filter(i => i.title === title)
        .sort((a, b) => b.created.localeCompare(a.created))[0];
      if (newItem) {
        selectedId = newItem.id;
        document.querySelectorAll('#items-body tr').forEach(r => r.classList.remove('selected'));
        const tr = document.querySelector(`#items-body tr[data-id="${CSS.escape(newItem.id)}"]`);
        if (tr) {
          tr.classList.add('selected');
          tr.scrollIntoView({ block: 'nearest' });
        }
        renderDetail(newItem);
        view.focus();
      }
    } catch (e) {
      showError(`Create failed: ${e}`);
    }
  }
  ```

- [ ] **Step 2: Manual test**

  In `just gui-dev`:
  - Click `+ New`, enter a title, press Enter or click Create
  - Expected: modal closes, new item is highlighted in the list (scrolled into view if needed), detail pane opens, CM6 editor is focused
  - Type body text immediately — should save correctly on blur/Cmd+S

- [ ] **Step 3: Commit**

  ```bash
  git add crumbs-gui/main.js
  git mit es
  git commit -m "Feat: auto-select and focus new item after creation in GUI (cr-4eq)"
  ```

---

## Chunk 4: Final verification and wrap-up

### Task 8: Regression pass, close crumbs, install

**Files:**
- No code changes expected

- [ ] **Step 1: Verify justfile dist tasks need no changes**

  Confirm `crumbs-gui/dist/codemirror.bundle.js` is present and committed. The justfile copies `{index.html,main.js,style.css}` — the bundle lives in `dist/` permanently (same as `marked.min.js`) so no justfile edits are needed. Run `just gui-dev` as a smoke test (faster than `just gui-build` which does a full native compile).

- [ ] **Step 2: Full regression pass in `just gui-dev`**

  Exercise all existing GUI functionality:
  - Store sidebar: switch stores, add/remove store
  - Create, update, close, delete items
  - Drag-and-drop rows between stores
  - Filter by status, priority, type, tag
  - Full-text search (debounced)
  - Defer modal, timer modal (start/stop), blocked-by modal
  - Inline title rename (double-click)
  - Export (JSON / CSV / TOON)
  - Emoji insertion
  - Preview mode toggle
  - Next button
  - Reindex, Refresh
  - Light/dark theme toggle (verify CM6 editor adapts)

- [ ] **Step 3: Close the crumbs items**

  ```bash
  crumbs close cr-4eq --reason "implemented: auto-select and focus after create"
  crumbs close cr-8r8 --reason "implemented: CM6 editor with line numbers, syntax highlight, search, outline panel"
  ```

- [ ] **Step 4: Install the updated GUI**

  ```bash
  just gui-install
  ```
