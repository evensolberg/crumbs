---
id: cr-r8q
title: 'Consider the option to add emoji to the body of a crumb. Maybe using :emoji: format?'
status: closed
type: task
priority: 2
tags: []
created: 2026-03-08
updated: 2026-03-08
closed_reason: 'Implemented in v0.13.0: :shortcode: expansion at write time + GUI emoji picker'
dependencies: []
---

# Consider the option to add emoji to the body of a crumb. Maybe using :emoji: format?

# cr-r8q: Emoji Shortcode Support + GUI Emoji Picker

## Context

Users want to embed emoji in crumb item bodies. The proposal is `:emoji:` shortcode
format (familiar from GitHub/Slack). Additionally, the GUI should have an emoji picker
button next to the **Preview** button in the detail pane — useful on Linux/Windows
where no OS-level emoji picker exists (macOS has `Ctrl+Cmd+Space`).

---

## Part 1: Rust — Write-time Shortcode Expansion

Convert `:shortcode:` to real Unicode **at write time** so files always store actual
emoji. Every consumer (CLI `show`, exports, GUI preview, editors) sees real emoji
without any per-consumer adaptation.

### 1.1 Dependency

`crumbs/Cargo.toml` → add:
```toml
emojis = "0.8"
```

### 1.2 New file: `crumbs/src/emoji.rs`

```rust
pub fn expand_shortcodes(text: &str) -> std::borrow::Cow<'_, str>
```

- Returns `Cow::Borrowed` when no shortcode found (zero allocation)
- Returns `Cow::Owned` when replacement occurs
- Skips fenced code blocks (` ``` ` / `~~~`) and inline backtick spans
- Allowed shortcode chars: `[a-zA-Z0-9_+\-]`, min 1 char, max 64
- Uses `emojis::get_by_shortcode(name)` → `.as_str()` for lookup
- Unknown shortcodes pass through unchanged (`:notreal:` stays as-is)
- No `regex` crate needed — plain character scan avoids the dependency

Unit tests (in `#[cfg(test)]` block in `emoji.rs`):
- `known_shortcode_expands` — `:smile:` → `"😄"`
- `unknown_shortcode_preserved` — `:notarealcode:` unchanged
- `no_shortcodes_returns_borrowed` — zero-alloc path
- `multiple_shortcodes` — `:tada: done :white_check_mark:` → `"🎉 done ✅"`
- `fenced_code_block_preserved` — `` ```\n:smile:\n``` `` unchanged
- `inline_code_preserved` — `` `:smile:` `` unchanged
- `partial_colon_not_expanded` — `:smile` (no closing colon) unchanged
- `empty_colons_not_expanded` — `::` unchanged
- `plus_one_shortcode` — `:+1:` → `"👍"`

### 1.3 `crumbs/src/lib.rs`

Add: `pub mod emoji;`

### 1.4 Write-path hookup (4 locations)

| File | Where | What |
|------|-------|-------|
| `commands/create.rs` | Top of `run()` | `let description = crate::emoji::expand_shortcodes(&description).into_owned();` |
| `commands/update.rs` | After `desc` is fully resolved (after the append/replace match block) | `let desc = crate::emoji::expand_shortcodes(&desc).into_owned();` |
| `commands/start.rs` | On `comment` before building entry line | wrap with `expand_shortcodes` |
| `commands/stop.rs` | Same as start.rs | Same pattern |

The Tauri `update_body` command calls `update::run` → covered automatically.
Do NOT apply in: `defer`, `close`, `link`, `block`, `move_`, `import` (no user body text).

### 1.5 Integration tests — `crumbs/tests/commands.rs`

Add 3 tests following existing style:
- `emoji_shortcodes_expanded_on_create` — create with `:tada:`, assert description == `"🎉"`
- `emoji_shortcodes_expanded_on_update_message` — update `--message ":bug: found"`, assert `"🐛 found"`
- `emoji_shortcodes_expanded_on_update_append` — append `:white_check_mark: fixed`, assert contains `"✅ fixed"`

---

## Part 2: GUI — Emoji Picker Button

A 😀 button placed **immediately after the Preview button** in `#detail-right .panel-title`
opens a floating emoji picker. Clicking an emoji inserts it at the cursor in `#detail-text`
and closes the picker. Pure vanilla JS/CSS — no external library.

### 2.1 `index.html` changes

In `#detail-right .panel-title` (currently lines 131–133), add after the Preview button:

```html
<button type="button" id="preview-btn" class="btn btn-secondary btn-small"
        title="Toggle markdown preview">Preview</button>
<button type="button" id="emoji-btn" class="btn btn-secondary btn-small"
        title="Insert emoji" disabled>😀</button>
<div id="emoji-picker" class="emoji-picker hidden"></div>
```

### 2.2 `main.js` changes

**A. `EMOJI_DATA` constant** — array of `{ cat, icon, emoji: [[shortcode, char], ...] }`
objects. ~200 entries across 8 categories (Smileys, People, Animals, Food, Travel,
Objects, Symbols, Flags). Built from this data: `EMOJI_LOOKUP` Map is derived at startup
for O(1) shortcode lookup.

**B. `expandEmoji(text)` function** — used in the GUI preview pre-pass:
```javascript
function expandEmoji(text) {
  return text.replace(/:([a-zA-Z0-9_+\-]+):/g, (m, n) => EMOJI_LOOKUP.get(n) ?? m);
}
```

**C. `buildEmojiPicker()` function** — called lazily on first open:
- Renders category tabs (icon only) + emoji grid
- Each emoji is a `<button class="ep-emoji">` with tooltip = shortcode name
- Click on emoji → `insertAtCursor(detailText, char)` → close picker

**D. `insertAtCursor(el, text)` helper**:
```javascript
function insertAtCursor(el, text) {
  const s = el.selectionStart, e = el.selectionEnd;
  el.value = el.value.slice(0, s) + text + el.value.slice(e);
  el.selectionStart = el.selectionEnd = s + text.length;
  el.dispatchEvent(new Event('input'));  // triggers autosave debounce
  el.focus();
}
```

**E. Picker toggle** — `emojiBtn.addEventListener('click', ...)`:
- Builds picker lazily on first open
- Toggles `hidden` on `#emoji-picker`
- `document` click-outside listener closes it (checking `!emojiBtn.contains(e.target)
  && !picker.contains(e.target)`)

**F. Enable/disable** — `emojiBtn` follows same pattern as `previewBtn`: enabled
when `selectedId` is non-null. Add to wherever `previewBtn` is enabled/disabled.

**G. Preview pre-pass** — change the `marked.parse` call:
```javascript
detailPreview.innerHTML = marked.parse(expandEmoji(detailText.value || ''));
```

### 2.3 `style.css` changes

```css
.emoji-picker {
  position: absolute;
  z-index: 200;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px;
  width: 320px;
  max-height: 260px;
  overflow-y: auto;
  box-shadow: 0 4px 12px rgba(0,0,0,0.25);
}
.emoji-picker .ep-tabs  { display: flex; gap: 2px; flex-wrap: wrap; margin-bottom: 4px; }
.emoji-picker .ep-tab   { font-size: 16px; background: none; border: none;
                          cursor: pointer; padding: 2px 5px; border-radius: 3px; }
.emoji-picker .ep-tab:hover, .emoji-picker .ep-tab.active { background: var(--accent-muted); }
.emoji-picker .ep-grid  { display: flex; flex-wrap: wrap; gap: 2px; }
.emoji-picker .ep-emoji { font-size: 20px; background: none; border: none;
                          cursor: pointer; padding: 3px; border-radius: 3px; }
.emoji-picker .ep-emoji:hover { background: var(--accent-muted); }
```

### 2.4 dist/ sync

After editing source files, copy to dist:
```
cp crumbs-gui/main.js crumbs-gui/dist/main.js
cp crumbs-gui/style.css crumbs-gui/dist/style.css
cp crumbs-gui/index.html crumbs-gui/dist/index.html
```

---

## Files to Change

| File | Change |
|------|--------|
| `crumbs/Cargo.toml` | Add `emojis = "0.8"` |
| `crumbs/src/emoji.rs` | **New**: `expand_shortcodes()` + unit tests |
| `crumbs/src/lib.rs` | Add `pub mod emoji;` |
| `crumbs/src/commands/create.rs` | Expand shortcodes on description |
| `crumbs/src/commands/update.rs` | Expand shortcodes on resolved desc |
| `crumbs/src/commands/start.rs` | Expand shortcodes on comment |
| `crumbs/src/commands/stop.rs` | Expand shortcodes on comment |
| `crumbs/tests/commands.rs` | Add 3 integration tests |
| `crumbs-gui/index.html` | Add emoji button + picker div after Preview button |
| `crumbs-gui/main.js` | EMOJI_DATA, expandEmoji, buildEmojiPicker, insertAtCursor |
| `crumbs-gui/style.css` | `.emoji-picker` styles |
| `crumbs-gui/dist/{index.html,main.js,style.css}` | Sync from source |

---

## Verification

1. `cargo nextest run -p crumbs` — all tests pass (existing + 9 new unit + 3 integration)
2. CLI smoke test:
   ```sh
   crumbs c 'Test' -m ':tada: done :white_check_mark:'
   crumbs show <id>   # body shows: 🎉 done ✅
   crumbs update <id> --append ':+1: looks good'
   crumbs show <id>   # new line shows: 👍 looks good
   ```
3. `just gui-dev` — launch GUI
4. Select an item → 😀 button enabled → click → picker opens next to Preview button
5. Click a category tab → grid shows that category's emoji
6. Click an emoji → inserted at cursor in textarea → picker closes
7. Toggle Preview → emoji shortcodes in body render as actual emoji
8. Click outside picker → picker closes
