import {
  EditorView, keymap, lineNumbers, highlightActiveLine,
  highlightActiveLineGutter, drawSelection, dropCursor,
  highlightSpecialChars, placeholder,
} from './codemirror.bundle.js';
import { EditorState, Compartment } from './codemirror.bundle.js';
import { defaultKeymap, history, historyKeymap, indentWithTab, deleteLine, moveLineUp, moveLineDown } from './codemirror.bundle.js';
import { closeBrackets, closeBracketsKeymap, snippet } from './codemirror.bundle.js';
import { search, searchKeymap, highlightSelectionMatches, openSearchPanel } from './codemirror.bundle.js';
import { syntaxHighlighting, defaultHighlightStyle, HighlightStyle } from './codemirror.bundle.js';
import { tags } from './codemirror.bundle.js';
import { markdown } from './codemirror.bundle.js';

const headingHighlight = HighlightStyle.define([
  { tag: tags.heading,  textDecoration: 'none', fontWeight: 'bold' },
  { tag: tags.heading1, color: 'var(--cm-h1)', fontWeight: 'bold' },
  { tag: tags.heading2, color: 'var(--cm-h2)' },
  { tag: tags.heading3, color: 'var(--cm-h3)' },
  { tag: tags.heading4, color: 'var(--cm-h4)' },
  { tag: tags.heading5, color: 'var(--cm-h5)' },
  { tag: tags.heading6, color: 'var(--cm-h6)' },
]);

const { invoke } = globalThis.__TAURI__.core;

// ── State ─────────────────────────────────────────────────────────────────

let storeDir = '';
let allItems = [];
let selectedId = null;
let outlineVisible = localStorage.getItem('outlineVisible') === 'true';
let lineWrapOn = localStorage.getItem('lineWrap') !== 'false'; // default on
const lineWrapCompartment = new Compartment();
let filterStatus   = 'all';
let filterPriority = 'any';
let filterType     = 'any';
let filterTag      = '';
let previewMode = false;
let pendingCloseId = '';
let autosaveTimer = null;
let timerInterval = null;
let loadedBody = '';
let sortCol = 'priority';
let sortDir = 'asc';
let searchResults = null;   // null = normal mode; Item[] = search mode
let searchTimer = null;
let dragItemId = null;      // ID of the row currently being dragged

// ── Column definitions ─────────────────────────────────────────────────────

const ALL_COLUMNS = [
  { key: 'id',           label: 'ID',       width: '90px',  sortable: true,  default: true  },
  { key: 'title',        label: 'Title',    width: null,    sortable: true,  default: true  },
  { key: 'status',       label: 'Status',   width: '110px', sortable: true,  default: true  },
  { key: 'type',         label: 'Type',     width: '70px',  sortable: true,  default: true  },
  { key: 'priority',     label: 'Priority', width: '110px', sortable: true,  default: true  },
  { key: 'due',          label: 'Due',      width: '90px',  sortable: true,  default: true  },
  { key: 'tags',         label: 'Tags',     width: '140px', sortable: true,  default: true  },
  { key: 'story_points', label: 'SP',       width: '44px',  sortable: true,  default: false },
  { key: 'created',      label: 'Created', width: '90px',  sortable: true,  default: false },
  { key: 'updated',      label: 'Updated', width: '90px',  sortable: true,  default: false },
];

function getVisibleColumns() {
  const saved = JSON.parse(localStorage.getItem('crumbs_visible_cols') || 'null');
  if (saved && Array.isArray(saved)) return saved;
  return ALL_COLUMNS.filter(c => c.default).map(c => c.key);
}
function saveVisibleColumns(keys) {
  localStorage.setItem('crumbs_visible_cols', JSON.stringify(keys));
}
let visibleCols = getVisibleColumns();

// ── DOM refs ──────────────────────────────────────────────────────────────

const statusStripCount  = document.getElementById('status-strip-count');
const statusStripBadges = document.getElementById('status-strip-badges');

const sidebarEl        = document.getElementById('sidebar');
const sidebarResizer   = document.getElementById('sidebar-resizer');
const sidebarToggleBtn = document.getElementById('sidebar-toggle-btn');
const sidebarAddBtn    = document.getElementById('sidebar-add-btn');
const sidebarRemoveBtn = document.getElementById('sidebar-remove-btn');
const storeListEl      = document.getElementById('store-list');
const storePathEl      = document.getElementById('store-path');
const showClosedEl     = document.getElementById('show-closed');
const themeBtn         = document.getElementById('theme-btn');
const refreshBtn       = document.getElementById('refresh-btn');
const errorBanner      = document.getElementById('error-banner');
const emptyState       = document.getElementById('empty-state');
const itemsTable       = document.getElementById('items-table');
const itemsBody        = document.getElementById('items-body');
const detailPane       = document.getElementById('detail-pane');
const detailLeft       = document.getElementById('detail-left');
const detailResizer    = document.getElementById('detail-resizer');
const resizeHandle     = document.getElementById('resize-handle');
const propGrid         = document.getElementById('prop-grid');
const detailActions    = document.getElementById('detail-actions');
const detailTitleLabel = document.getElementById('detail-title-label');
const formatToolbar    = document.getElementById('format-toolbar');
const headingSelect    = document.getElementById('heading-select');
const fmtWrapBtn       = document.getElementById('fmt-wrap');
const detailEditorEl   = document.getElementById('detail-editor');
const outlinePanel     = document.getElementById('outline-panel');
const outlineList      = document.getElementById('outline-list');
const outlineToggleBtn = document.getElementById('outline-toggle-btn');
const detailPreview    = document.getElementById('detail-preview');
const previewBtn       = document.getElementById('preview-btn');
const emojiBtn         = document.getElementById('emoji-btn');
const emojiPicker      = document.getElementById('emoji-picker');

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

// ── CodeMirror editor instance ─────────────────────────────────────────────
const view = new EditorView({
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
      syntaxHighlighting(headingHighlight),
      markdown(),
      closeBrackets(),
      search({ top: false }),
      highlightSelectionMatches(),
      lineWrapCompartment.of(lineWrapOn ? EditorView.lineWrapping : []),
      placeholder('No body text.'),
      keymap.of([
        ...closeBracketsKeymap,
        ...defaultKeymap,
        ...historyKeymap,
        ...searchKeymap,
        indentWithTab,
        { key: 'Mod-s',          run: () => { flushAutosave(); return true; } },
        { key: 'Mod-b',          run: () => { wrapInline('**'); return true; } },
        { key: 'Mod-i',          run: () => { wrapInline('*');  return true; } },
        { key: 'Mod-d',          run: deleteLine },
        { key: 'Mod-ArrowUp',    run: moveLineUp },
        { key: 'Mod-ArrowDown',  run: moveLineDown },
        { key: 'Mod-0',          run: () => { applyHeading(0); return true; } },
        { key: 'Mod-1',          run: () => { applyHeading(1); return true; } },
        { key: 'Mod-2',          run: () => { applyHeading(2); return true; } },
        { key: 'Mod-3',          run: () => { applyHeading(3); return true; } },
        { key: 'Mod-4',          run: () => { applyHeading(4); return true; } },
        { key: 'Mod-5',          run: () => { applyHeading(5); return true; } },
        { key: 'Mod-6',          run: () => { applyHeading(6); return true; } },
        { key: 'Mod-k',          run: snippet('[${text}](${url})') },
      ]),
      EditorView.updateListener.of(update => {
        if (update.docChanged) {
          scheduleAutosave();
          scheduleOutlineUpdate(); // defined below; hoisted as a function declaration
        }
      }),
      EditorView.domEventHandlers({
        blur: () => { flushAutosave(); },
      }),
    ],
  }),
  parent: detailEditorEl,
});

// Apply initial outline visibility
outlinePanel.classList.toggle('hidden', !outlineVisible);
outlineToggleBtn.classList.toggle('active', outlineVisible);

// Toolbar action buttons
const newBtn        = document.getElementById('new-btn');
const helpBtn       = document.getElementById('help-btn');
const nextBtn       = document.getElementById('next-btn');
const startBtn      = document.getElementById('start-btn');
const blockBtn      = document.getElementById('block-btn');
const deferBtn      = document.getElementById('defer-btn');
const timerBtn      = document.getElementById('timer-btn');
const closeItemBtn  = document.getElementById('close-item-btn');
const deleteBtn     = document.getElementById('delete-btn');
const cleanBtn      = document.getElementById('clean-btn');
const exportBtn     = document.getElementById('export-btn');
const reindexBtn    = document.getElementById('reindex-btn');
const searchInput   = document.getElementById('search-input');

// Export modal
const exportModal      = document.getElementById('export-modal');
const exportCancelBtn  = document.getElementById('export-cancel-btn');
const exportConfirmBtn = document.getElementById('export-confirm-btn');

// Delete modal
const deleteModal      = document.getElementById('delete-modal');
const deleteCancelBtn  = document.getElementById('delete-cancel-btn');
const deleteConfirmBtn = document.getElementById('delete-confirm-btn');

// Close modal
const closeModal      = document.getElementById('close-modal');
const closeReason     = document.getElementById('close-reason');
const closeCancelBtn  = document.getElementById('close-cancel-btn');
const closeConfirmBtn = document.getElementById('close-confirm-btn');

// New item modal
const newModal      = document.getElementById('new-modal');
const helpModal     = document.getElementById('help-modal');
const newTitle      = document.getElementById('new-title');
const newCancelBtn  = document.getElementById('new-cancel-btn');
const newConfirmBtn = document.getElementById('new-confirm-btn');

// Blocked-by modal
const blockedByModal     = document.getElementById('blocked-by-modal');
const blockerTargetTitle = document.getElementById('blocker-target-title');
const newBlockerTitle    = document.getElementById('new-blocker-title');
const newBlockerBtn      = document.getElementById('new-blocker-btn');
const blockerSearch      = document.getElementById('blocker-search');
const blockerList        = document.getElementById('blocker-list');
const blockerCancelBtn   = document.getElementById('blocker-cancel-btn');
const blockerConfirmBtn  = document.getElementById('blocker-confirm-btn');

// Defer modal
const deferModal      = document.getElementById('defer-modal');
const deferUntil      = document.getElementById('defer-until');
const deferCancelBtn  = document.getElementById('defer-cancel-btn');
const deferConfirmBtn = document.getElementById('defer-confirm-btn');

// Timer modal
const timerModal        = document.getElementById('timer-modal');
const timerModalTitle   = document.getElementById('timer-modal-title');
const timerComment      = document.getElementById('timer-comment');
const timerCancelBtn    = document.getElementById('timer-cancel-btn');
const timerConfirmBtn   = document.getElementById('timer-confirm-btn');

// Open Dir modal
const openDirModal      = document.getElementById('open-dir-modal');
const dirPathInput      = document.getElementById('dir-path-input');
const dirPickBtn        = document.getElementById('dir-pick-btn');
const initPrompt        = document.getElementById('init-prompt');
const initNoBtn         = document.getElementById('init-no-btn');
const initYesBtn        = document.getElementById('init-yes-btn');
const openDirActions    = document.getElementById('open-dir-actions');
const openDirCancelBtn  = document.getElementById('open-dir-cancel-btn');
const openDirOkBtn      = document.getElementById('open-dir-ok-btn');

// ── Helpers ───────────────────────────────────────────────────────────────

function showError(msg) {
  errorBanner.textContent = msg;
  errorBanner.classList.remove('hidden');
}
function clearError() {
  errorBanner.classList.add('hidden');
}

function escHtml(s) {
  return String(s ?? '').replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
}

const PRIORITY_LABELS = ['critical', 'high', 'normal', 'low', 'backlog'];
const STATUS_LABELS   = {
  open: 'Open',
  in_progress: 'In Progress',
  blocked: 'Blocked',
  deferred: 'Deferred',
  closed: 'Closed',
};

function statusBadge(status) {
  return `<span class="badge badge-${status}">${STATUS_LABELS[status] ?? status}</span>`;
}

function priorityBadge(p) {
  const label = PRIORITY_LABELS[p] ?? '';
  return `<span class="p-badge p${p}">P${p} <span class="p-badge-label">${label.charAt(0).toUpperCase() + label.slice(1)}</span></span>`;
}

function dueHtml(due) {
  if (!due) return '';
  const today = new Date().toISOString().slice(0, 10);
  return due < today
    ? `<span class="due-overdue">!${due}</span>`
    : `<span class="due-normal">${due}</span>`;
}

function selectedItem() {
  return allItems.find(i => i.id === selectedId) ?? null;
}

// Returns true when any interactive control has keyboard focus, so global
// shortcuts are suppressed and don't fire unexpectedly.
function isControlFocused() {
  const el = document.activeElement;
  if (!el) return false;
  const tag = el.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  if (el.closest('.cm-editor')) return true;
  // Treat any other focusable element (button, link, etc.) as "focused"
  // so global shortcuts don't fire when a toolbar control has keyboard focus.
  if (tag === 'BUTTON' || tag === 'A') return true;
  if (el.isContentEditable) return true;
  if (el.getAttribute('role') === 'textbox') return true;
  return false;
}

function isModalOpen() {
  return !!document.querySelector('.modal:not(.hidden)');
}

function selectRow(id, tr) {
  selectedId = id;
  for (const r of document.querySelectorAll('#items-body tr.selected')) r.classList.remove('selected');
  if (!tr) tr = document.querySelector(`#items-body tr[data-id="${CSS.escape(id)}"]`);
  if (tr) {
    tr.classList.add('selected');
    tr.scrollIntoView({ block: 'nearest' });
  }
  renderDetail(selectedItem());
}

// ── Toolbar contextual button state ──────────────────────────────────────

function hasActiveTimer(description) {
  if (!description) return false;
  let active = false;
  for (const line of description.split('\n')) {
    const t = line.trim();
    if (t.startsWith('[start]')) active = true;
    else if (t.startsWith('[stop]')) active = false;
  }
  return active;
}

function updateToolbarButtons() {
  const item = selectedItem();
  const hasSelection = item !== null;
  const isClosed = item?.status === 'closed';
  const timerActive = hasActiveTimer(item?.description);

  startBtn.disabled    = !hasSelection || isClosed || item?.status === 'in_progress';
  blockBtn.disabled    = !hasSelection || isClosed;
  deferBtn.disabled    = !hasSelection || isClosed || item?.status === 'deferred';
  timerBtn.disabled    = !hasSelection || isClosed;
  timerBtn.textContent = timerActive ? '■ Stop' : '▶ Timer';
  timerBtn.title       = timerActive ? 'Stop the active timer' : 'Start a time-tracking timer';
  closeItemBtn.disabled = !hasSelection || isClosed;
  deleteBtn.disabled   = !hasSelection;
  emojiBtn.disabled    = !hasSelection;
}

// ── Vertical resize ───────────────────────────────────────────────────────

(function initResize() {
  let dragging = false;
  let startY = 0;
  let startDetailH = 0;

  resizeHandle.addEventListener('mousedown', e => {
    dragging = true;
    startY = e.clientY;
    startDetailH = detailPane.getBoundingClientRect().height;
    resizeHandle.classList.add('dragging');
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'row-resize';
  });

  document.addEventListener('mousemove', e => {
    if (!dragging) return;
    const delta = e.clientY - startY;
    const newH = Math.max(80, startDetailH - delta);
    detailPane.style.flex = `0 0 ${newH}px`;
  });

  document.addEventListener('mouseup', () => {
    if (!dragging) return;
    dragging = false;
    resizeHandle.classList.remove('dragging');
    document.body.style.userSelect = '';
    document.body.style.cursor = '';
  });
})();

// ── Table rendering ───────────────────────────────────────────────────────

function filteredItems() {
  return allItems.filter(item => {
    if (filterStatus !== 'all' && item.status !== filterStatus) return false;
    if (filterPriority !== 'any' && String(item.priority) !== filterPriority) return false;
    if (filterType !== 'any' && (item.type ?? 'task') !== filterType) return false;
    if (filterTag && !(item.tags ?? []).some(t => t.toLowerCase().includes(filterTag.toLowerCase()))) return false;
    return true;
  });
}

function sortedItems() {
  const items = searchResults !== null ? searchResults : filteredItems();
  const dir = sortDir === 'asc' ? 1 : -1;
  return items.slice().sort((a, b) => {
    let av, bv;
    switch (sortCol) {
      case 'id':       av = a.id;       bv = b.id;       break;
      case 'title':    av = a.title;    bv = b.title;    break;
      case 'status':   av = a.status;   bv = b.status;   break;
      case 'type':     av = a.type ?? ''; bv = b.type ?? ''; break;
      case 'priority': av = a.priority; bv = b.priority; break;
      case 'due':      av = a.due ?? '9999'; bv = b.due ?? '9999'; break;
      case 'tags':         av = (a.tags ?? []).join(); bv = (b.tags ?? []).join(); break;
      case 'story_points': av = a.story_points ?? 999; bv = b.story_points ?? 999; break;
      case 'created':      av = a.created ?? ''; bv = b.created ?? ''; break;
      case 'updated':      av = a.updated ?? ''; bv = b.updated ?? ''; break;
      default:             av = a.priority; bv = b.priority;
    }
    if (av < bv) return -dir;
    if (av > bv) return  dir;
    return 0;
  });
}

function activeTimerStart(desc) {
  if (!desc) return null;
  let ts = null;
  for (const line of desc.split('\n')) {
    const t = line.trim();
    if (t.startsWith('[start]')) ts = t.slice(7).trim().slice(0, 19);
    else if (t.startsWith('[stop]')) ts = null;
  }
  return ts;
}

function totalTrackedSecs(desc) {
  if (!desc) return 0;
  let total = 0;
  let startMs = null;
  for (const line of desc.split('\n')) {
    const t = line.trim();
    if (t.startsWith('[start]')) {
      const ts = t.slice(7).trim().slice(0, 19).replace(' ', 'T');
      startMs = new Date(ts).getTime();
    } else if (t.startsWith('[stop]') && startMs !== null) {
      const ts = t.slice(6).trim().slice(0, 19).replace(' ', 'T');
      total += Math.max(0, (new Date(ts).getTime() - startMs) / 1000);
      startMs = null;
    }
  }
  return total;
}

function formatElapsed(secs) {
  secs = Math.max(0, Math.floor(secs));
  if (secs < 60)   return `${secs}s`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
  return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m ${secs % 60}s`;
}

function cellFor(item, colKey) {
  switch (colKey) {
    case 'id':           return `<td class="item-id">${escHtml(item.id)}</td>`;
    case 'title': {
      const prefix = hasActiveTimer(item.description)
        ? '<span style="color:var(--accent)" title="Timer running">▶</span> '
        : '';
      return `<td class="item-title">${prefix}${escHtml(item.title)}</td>`;
    }
    case 'status':       return `<td>${statusBadge(item.status)}</td>`;
    case 'type': {
      const typeColor = { bug: 'var(--type-bug)', feature: 'var(--type-feature)', epic: 'var(--type-epic)', idea: 'var(--type-idea)' }[item.type] ?? 'var(--text-dim)';
      return `<td style="font-size:11px;color:${typeColor};font-weight:${item.type === 'bug' ? '600' : 'normal'}">${escHtml(item.type ?? '')}</td>`;
    }
    case 'priority':     return `<td>${priorityBadge(item.priority)}</td>`;
    case 'due':          return `<td>${dueHtml(item.due)}</td>`;
    case 'tags':         return `<td class="item-tags">${escHtml((item.tags ?? []).join(', '))}</td>`;
    case 'story_points': return `<td style="text-align:center;font-size:11px;color:var(--text-dim)">${item.story_points != null ? item.story_points : '—'}</td>`;
    case 'created':      return `<td style="font-size:11px;color:var(--text-dim)">${escHtml(item.created ?? '')}</td>`;
    case 'updated':      return `<td style="font-size:11px;color:var(--text-dim)">${escHtml(item.updated ?? '')}</td>`;
    default:             return '<td></td>';
  }
}

function rebuildTableHeader() {
  const colgroup = document.getElementById('items-colgroup');
  const theadRow = document.getElementById('items-thead-row');
  colgroup.innerHTML = '';
  theadRow.innerHTML = '';
  for (const key of visibleCols) {
    const col = ALL_COLUMNS.find(c => c.key === key);
    if (!col) continue;
    const colEl = document.createElement('col');
    if (col.width) colEl.style.width = col.width;
    colgroup.appendChild(colEl);
    const th = document.createElement('th');
    th.dataset.col = key;
    th.textContent = col.label;
    const arrow = document.createElement('span');
    arrow.className = 'sort-arrow';
    th.appendChild(arrow);
    th.addEventListener('click', () => {
      if (sortCol === key) { sortDir = sortDir === 'asc' ? 'desc' : 'asc'; }
      else { sortCol = key; sortDir = 'asc'; }
      updateSortHeaders();
      renderTable();
    });
    theadRow.appendChild(th);
  }
  updateSortHeaders();
}

function updateSortHeaders() {
  for (const th of document.querySelectorAll('#items-thead-row th[data-col]')) {
    th.classList.remove('sort-asc', 'sort-desc');
    if (th.dataset.col === sortCol) {
      th.classList.add(sortDir === 'asc' ? 'sort-asc' : 'sort-desc');
    }
  }
}

const STRIP_STATS = [
  { key: 'open',        label: 'open',        color: 'var(--status-open)' },
  { key: 'in_progress', label: 'in progress',  color: 'var(--status-in-progress)' },
  { key: 'blocked',     label: 'blocked',      color: 'var(--status-blocked)' },
  { key: 'deferred',    label: 'deferred',     color: 'var(--status-deferred)' },
  { key: 'closed',      label: 'closed',       color: 'var(--status-closed)' },
];

function updateStatusStrip(filtered) {
  const total = allItems.length;
  if (searchResults !== null) {
    const q = searchInput.value.trim();
    statusStripCount.textContent = `${filtered.length} result${filtered.length !== 1 ? 's' : ''} for "${q}"`;
  } else {
    statusStripCount.textContent = filtered.length === total
      ? `${total} item${total !== 1 ? 's' : ''}`
      : `Showing ${filtered.length} of ${total}`;
  }

  const counts = {};
  for (const item of filtered) counts[item.status] = (counts[item.status] ?? 0) + 1;

  statusStripBadges.innerHTML = STRIP_STATS
    .filter(s => counts[s.key])
    .map(s => `<span class="strip-stat">
      <span class="strip-dot" style="background:${s.color}"></span>
      ${counts[s.key]} ${s.label}
    </span>`)
    .join('');
}

function renderTable() {
  const items = sortedItems();
  updateStatusStrip(items);
  itemsBody.innerHTML = '';

  if (items.length === 0) {
    itemsTable.classList.add('hidden');
    emptyState.classList.remove('hidden');
    return;
  }

  emptyState.classList.add('hidden');
  itemsTable.classList.remove('hidden');

  for (const item of items) {
    const tr = document.createElement('tr');
    if (item.id === selectedId) tr.classList.add('selected');
    tr.innerHTML = visibleCols.map(key => cellFor(item, key)).join('');
    tr.dataset.id = item.id;
    tr.addEventListener('mousedown', e => startRowDrag(e, item, tr));
    itemsBody.appendChild(tr);
  }
}

// ── Column sorting + resizing (dynamic, called after rebuildTableHeader) ──

function initColResizers() {
  const cols = document.querySelectorAll('#items-colgroup col');
  const ths  = document.querySelectorAll('#items-thead-row th');

  ths.forEach((th, i) => {
    const handle = document.createElement('div');
    handle.className = 'col-resizer';
    th.appendChild(handle);

    let startX = 0;
    let startW = 0;

    handle.addEventListener('mousedown', e => {
      e.preventDefault();   // prevent text selection when dragging
      e.stopPropagation();
      const table = document.getElementById('items-table');
      // Snapshot all rendered widths first, then lock both <col> and <th>
      // so table-layout: fixed has no freedom to redistribute.
      // (<th> cell widths override <col> widths, so both must be pinned.)
      const widths = Array.from(ths).map(t => t.getBoundingClientRect().width);
      widths.forEach((w, j) => {
        cols[j].style.width = `${w}px`;
        ths[j].style.width  = `${w}px`;
      });
      const startTableW = table.getBoundingClientRect().width;
      table.style.minWidth = '0';   // prevent CSS min-width:100% from fighting explicit width
      table.style.width = `${startTableW}px`;
      startX = e.clientX;
      startW = widths[i];
      handle.classList.add('resizing');
      document.body.style.cursor = 'col-resize';

      function onMove(e) {
        e.preventDefault();
        const w = Math.max(40, startW + e.clientX - startX);
        cols[i].style.width  = `${w}px`;
        ths[i].style.width   = `${w}px`;
        table.style.width    = `${startTableW + (w - startW)}px`;
      }
      function onUp() {
        handle.classList.remove('resizing');
        document.body.style.cursor = '';
        document.removeEventListener('mousemove', onMove);
        document.removeEventListener('mouseup', onUp);
      }
      document.addEventListener('mousemove', onMove);
      document.addEventListener('mouseup', onUp);
    });
  });
}

// ── Detail pane helpers ───────────────────────────────────────────────────

function propRow(label, valueHtml) {
  const lEl = document.createElement('div');
  lEl.className = 'prop-label';
  lEl.textContent = label;
  const vEl = document.createElement('div');
  vEl.className = 'prop-value';
  vEl.innerHTML = valueHtml;
  propGrid.appendChild(lEl);
  propGrid.appendChild(vEl);
  return vEl;
}

function makeSelect(options, current, onChange) {
  const sel = document.createElement('select');
  for (const [val, lbl] of options) {
    const opt = document.createElement('option');
    opt.value = val;
    opt.textContent = lbl;
    if (val === current) opt.selected = true;
    sel.appendChild(opt);
  }
  sel.addEventListener('change', () => onChange(sel.value));
  return sel;
}

function renderProps(item) {
  if (timerInterval) { clearInterval(timerInterval); timerInterval = null; }
  propGrid.innerHTML = '';

  propRow('ID', escHtml(item.id));

  propRow('Status', '').appendChild(makeSelect(
    Object.entries(STATUS_LABELS),
    item.status,
    v => doUpdateStatus(item.id, v),
  ));

  propRow('Priority', '').appendChild(makeSelect(
    Array.from({ length: 5 }, (_, p) => [String(p), `P${p} — ${PRIORITY_LABELS[p]}`]),
    String(item.priority),
    v => doUpdatePriority(item.id, Number(v)),
  ));

  propRow('Type', '').appendChild(makeSelect(
    ['task', 'bug', 'feature', 'epic', 'idea'].map(t => [t, t]),
    item.type ?? '',
    v => doUpdateType(item.id, v),
  ));

  propRow('Points', '').appendChild(makeSelect(
    [['0', '—'], ['1', '1'], ['2', '2'], ['3', '3'], ['5', '5'], ['8', '8'], ['13', '13'], ['21', '21']],
    String(item.story_points ?? 0),
    v => doUpdatePoints(item.id, Number(v)),
  ));

  propRow('Created', escHtml(item.created ?? ''));
  propRow('Updated', escHtml(item.updated ?? ''));

  const completedSecs = totalTrackedSecs(item.description ?? '');
  const startTs = activeTimerStart(item.description ?? '');
  if (startTs || completedSecs > 0) {
    const el = document.createElement('span');
    if (startTs) {
      const startMs = new Date(startTs.replace(' ', 'T')).getTime();
      const tick = () => {
        const liveSecs = Math.max(0, (Date.now() - startMs) / 1000);
        el.textContent = formatElapsed(completedSecs + liveSecs) + '  ▶';
      };
      tick();
      timerInterval = setInterval(tick, 1000);
    } else {
      el.textContent = formatElapsed(completedSecs);
    }
    propRow('Tracked', '').appendChild(el);
  }

  const dueInput = document.createElement('input');
  dueInput.type = 'date';
  dueInput.value = item.due ?? '';
  dueInput.addEventListener('change', () => doUpdateDue(item.id, dueInput.value));
  propRow('Due', '').appendChild(dueInput);

  const tagsInput = document.createElement('input');
  tagsInput.type = 'text';
  tagsInput.placeholder = 'comma, separated';
  tagsInput.value = (item.tags ?? []).join(', ');
  tagsInput.style.cssText = 'width:100%;font:inherit;background:var(--bg);color:var(--text);border:1px solid var(--border);border-radius:3px;padding:2px 4px;outline:none;box-sizing:border-box;';
  let loadedTags = tagsInput.value;
  tagsInput.addEventListener('focus', () => { tagsInput.style.borderColor = 'var(--accent)'; });
  tagsInput.addEventListener('blur', () => {
    tagsInput.style.borderColor = 'var(--border)';
    if (tagsInput.value !== loadedTags) {
      loadedTags = tagsInput.value;
      doUpdateTags(item.id, tagsInput.value);
    }
  });
  tagsInput.addEventListener('keydown', e => {
    if (e.key === 'Enter') tagsInput.blur();
    if (e.key === 'Escape') { tagsInput.value = loadedTags; tagsInput.blur(); e.stopPropagation(); }
  });
  propRow('Tags', '').appendChild(tagsInput);

  const depsInput = document.createElement('input');
  depsInput.type = 'text';
  depsInput.placeholder = 'id1, id2, …';
  depsInput.value = (item.dependencies ?? []).join(', ');
  depsInput.style.cssText = 'width:100%;font:inherit;background:var(--bg);color:var(--text);border:1px solid var(--border);border-radius:3px;padding:2px 4px;outline:none;box-sizing:border-box;';
  let loadedDeps = depsInput.value;
  depsInput.addEventListener('focus', () => { depsInput.style.borderColor = 'var(--accent)'; });
  depsInput.addEventListener('blur', () => {
    depsInput.style.borderColor = 'var(--border)';
    if (depsInput.value !== loadedDeps) {
      loadedDeps = depsInput.value;
      doUpdateDependencies(item.id, depsInput.value);
    }
  });
  depsInput.addEventListener('keydown', e => {
    if (e.key === 'Enter') depsInput.blur();
    if (e.key === 'Escape') { depsInput.value = loadedDeps; depsInput.blur(); e.stopPropagation(); }
  });
  propRow('Depends on', '').appendChild(depsInput);
  if ((item.blocks ?? []).length > 0) {
    propRow('Blocks', escHtml(item.blocks.join(', ')));
  }
  if ((item.blocked_by ?? []).length > 0) {
    propRow('Blocked by', escHtml(item.blocked_by.join(', ')));
  }
  if (item.closed_reason) {
    propRow('Reason', escHtml(item.closed_reason));
  }
}

function setPreviewMode(on) {
  previewMode = on;
  previewBtn.textContent = on ? 'Edit' : 'Preview';
  formatToolbar.classList.toggle('hidden', on);
  detailEditorEl.classList.toggle('hidden', on);
  outlinePanel.classList.toggle('hidden', on || !outlineVisible);
  detailPreview.classList.toggle('hidden', !on);
  if (on) {
    detailPreview.innerHTML = marked.parse(expandEmoji(view.state.doc.toString()));
  }
}

// ── Format toolbar ────────────────────────────────────────────────────────

function wrapInline(marker, closeMarker = marker) {
  const { state } = view;
  const changes = [];
  const newRanges = [];
  for (const range of state.selection.ranges) {
    if (range.empty) {
      changes.push({ from: range.from, insert: marker + closeMarker });
      newRanges.push({ anchor: range.from + marker.length });
    } else {
      const text = state.doc.sliceString(range.from, range.to);
      // Toggle: remove markers if already wrapped
      if (text.startsWith(marker) && text.endsWith(closeMarker) && text.length > marker.length + closeMarker.length) {
        changes.push({ from: range.from, to: range.to, insert: text.slice(marker.length, text.length - closeMarker.length) });
      } else {
        changes.push({ from: range.from, insert: marker });
        changes.push({ from: range.to, insert: closeMarker });
      }
    }
  }
  view.dispatch({ changes });
  view.focus();
}

function applyHeading(level) {
  const { state } = view;
  const changes = [];
  for (const range of state.selection.ranges) {
    const startLine = state.doc.lineAt(range.from);
    const endLine   = state.doc.lineAt(range.to);
    for (let n = startLine.number; n <= endLine.number; n++) {
      const line  = state.doc.line(n);
      const match = line.text.match(/^(#{1,6}) /);
      if (level === 0) {
        if (match) changes.push({ from: line.from, to: line.from + match[1].length + 1, insert: '' });
      } else {
        const prefix = '#'.repeat(level) + ' ';
        if (match) {
          changes.push({ from: line.from, to: line.from + match[1].length + 1, insert: prefix });
        } else {
          changes.push({ from: line.from, insert: prefix });
        }
      }
    }
  }
  if (changes.length) view.dispatch({ changes });
  // Sync the dropdown to reflect the applied level
  if (headingSelect) headingSelect.value = String(level);
  view.focus();
}

function toggleLinePrefix(prefix) {
  const { state } = view;
  const changes = [];
  for (const range of state.selection.ranges) {
    const startLine = state.doc.lineAt(range.from);
    const endLine   = state.doc.lineAt(range.to);
    for (let n = startLine.number; n <= endLine.number; n++) {
      const line = state.doc.line(n);
      if (line.text.startsWith(prefix)) {
        changes.push({ from: line.from, to: line.from + prefix.length, insert: '' });
      } else {
        changes.push({ from: line.from, insert: prefix });
      }
    }
  }
  if (changes.length) view.dispatch({ changes });
  view.focus();
}

function insertCodeBlock() {
  const { state } = view;
  const range = state.selection.main;
  if (range.empty) {
    const insert = '```\n\n```';
    view.dispatch({ changes: { from: range.from, insert }, selection: { anchor: range.from + 4 } });
  } else {
    const selected = state.doc.sliceString(range.from, range.to);
    view.dispatch({ changes: { from: range.from, to: range.to, insert: '```\n' + selected + '\n```' } });
  }
  view.focus();
}

function insertHR() {
  const { state } = view;
  const line = state.doc.lineAt(state.selection.main.from);
  const insert = line.text.trim() === '' ? '---' : '\n---';
  view.dispatch({ changes: { from: line.to, insert } });
  view.focus();
}

function toggleLineWrap() {
  lineWrapOn = !lineWrapOn;
  localStorage.setItem('lineWrap', lineWrapOn);
  view.dispatch({ effects: lineWrapCompartment.reconfigure(lineWrapOn ? EditorView.lineWrapping : []) });
  fmtWrapBtn.classList.toggle('active', lineWrapOn);
  view.focus();
}

// Sync heading dropdown to current cursor line on selection change
EditorView.updateListener.of(update => {
  if (update.selectionSet && headingSelect) {
    const line = update.state.doc.lineAt(update.state.selection.main.from);
    const m = line.text.match(/^(#{1,6}) /);
    headingSelect.value = m ? String(m[1].length) : '0';
  }
});

// Wire format toolbar buttons
headingSelect.addEventListener('change', () => { applyHeading(parseInt(headingSelect.value, 10)); });
document.getElementById('fmt-bold').addEventListener('click',      () => wrapInline('**'));
document.getElementById('fmt-italic').addEventListener('click',    () => wrapInline('*'));
document.getElementById('fmt-code').addEventListener('click',      () => wrapInline('`'));
document.getElementById('fmt-codeblock').addEventListener('click', () => insertCodeBlock());
document.getElementById('fmt-quote').addEventListener('click',     () => toggleLinePrefix('> '));
document.getElementById('fmt-hr').addEventListener('click',        () => insertHR());
document.getElementById('fmt-find').addEventListener('click',      () => { openSearchPanel(view); view.focus(); });
fmtWrapBtn.addEventListener('click', toggleLineWrap);
fmtWrapBtn.classList.toggle('active', lineWrapOn);

// ── Heading outline ────────────────────────────────────────────────────────

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

outlineToggleBtn.addEventListener('click', () => {
  outlineVisible = !outlineVisible;
  localStorage.setItem('outlineVisible', outlineVisible);
  outlinePanel.classList.toggle('hidden', !outlineVisible || previewMode);
  outlineToggleBtn.classList.toggle('active', outlineVisible);
  if (outlineVisible) renderOutline();
});

function renderDetail(item) {
  if (timerInterval) { clearInterval(timerInterval); timerInterval = null; }
  if (!item) {
    detailPane.classList.add('hidden');
    detailActions.innerHTML = '';
    updateToolbarButtons();
    return;
  }

  detailPane.classList.remove('hidden');
  detailTitleLabel.textContent = item.title;
  detailTitleLabel.title = 'Double-click to rename';
  detailTitleLabel.dataset.id = item.id;
  renderProps(item);
  detailActions.innerHTML = '';
  loadedBody = item.description ?? '';
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: loadedBody },
  });
  setPreviewMode(false);
  renderOutline();
  updateToolbarButtons();
}

// ── Data loading ──────────────────────────────────────────────────────────

async function loadItems() {
  clearError();
  try {
    allItems = await invoke('list_items', {
      dir: storeDir,
      includeClosed: showClosedEl.checked,
    });
    renderTable();
    renderDetail(selectedItem());
  } catch (e) {
    showError(`Failed to load items: ${e}`);
  }
}

// ── Update actions ────────────────────────────────────────────────────────

async function doUpdateStatus(id, status) {
  clearError();
  try {
    await invoke('update_status', { dir: storeDir, id, status });
    await loadItems();
  } catch (e) {
    showError(`Update failed: ${e}`);
  }
}

async function doUpdatePriority(id, priority) {
  clearError();
  try {
    await invoke('update_priority', { dir: storeDir, id, priority });
    await loadItems();
  } catch (e) {
    showError(`Update failed: ${e}`);
  }
}

async function doUpdateType(id, itemType) {
  clearError();
  try {
    await invoke('update_type', { dir: storeDir, id, itemType });
    await loadItems();
  } catch (e) {
    showError(`Update failed: ${e}`);
  }
}

async function doUpdatePoints(id, points) {
  clearError();
  try {
    await invoke('update_points', { dir: storeDir, id, points });
    await loadItems();
  } catch (e) {
    showError(`Update failed: ${e}`);
  }
}

async function doUpdateDue(id, due) {
  clearError();
  try {
    await invoke('update_due', { dir: storeDir, id, due });
    await loadItems();
  } catch (e) {
    showError(`Update failed: ${e}`);
  }
}

async function doUpdateDependencies(id, dependencies) {
  clearError();
  try {
    await invoke('update_dependencies', { dir: storeDir, id, dependencies });
    await loadItems();
  } catch (e) {
    showError(`Update failed: ${e}`);
  }
}

async function doUpdateTags(id, tags) {
  clearError();
  try {
    await invoke('update_tags', { dir: storeDir, id, tags });
    await loadItems();
  } catch (e) {
    showError(`Update failed: ${e}`);
  }
}

async function doUpdateTitle(id, title) {
  clearError();
  try {
    await invoke('update_title', { dir: storeDir, id, title });
    await loadItems();
  } catch (e) {
    showError(`Rename failed: ${e}`);
  }
}

async function doSaveText(id, text) {
  clearError();
  try {
    await invoke('update_body', { dir: storeDir, id, body: text });
    loadedBody = text;
    await loadItems();
  } catch (e) {
    showError(`Save failed: ${e}`);
  }
}

// ── Search ────────────────────────────────────────────────────────────────

async function doSearch(query) {
  const q = query.trim();
  if (!q) {
    searchResults = null;
    renderTable();
    return;
  }
  clearError();
  try {
    searchResults = await invoke('search_items', {
      dir: storeDir,
      query: q,
      includeClosed: showClosedEl.checked,
    });
    renderTable();
  } catch (e) {
    showError(`Search failed: ${e}`);
  }
}

searchInput.addEventListener('input', () => {
  clearTimeout(searchTimer);
  searchTimer = setTimeout(() => doSearch(searchInput.value), 350);
});
searchInput.addEventListener('search', () => {
  // fires when the × clear button is clicked on type="search"
  clearTimeout(searchTimer);
  doSearch(searchInput.value);
});

// ── Next ──────────────────────────────────────────────────────────────────

function doNext() {
  const today = new Date().toISOString().slice(0, 10);
  const open = allItems
    .filter(i => {
      if (i.status === 'closed') return false;
      if (i.status === 'deferred') return i.due ? i.due <= today : false;
      return true;
    })
    .sort((a, b) => a.priority - b.priority || a.created.localeCompare(b.created));
  if (!open.length) return;
  selectedId = open[0].id;
  renderTable();
  renderDetail(selectedItem());
  document.querySelector(`#items-body tr[data-id="${CSS.escape(selectedId)}"]`)
    ?.scrollIntoView({ block: 'nearest' });
}

// ── Export modal ──────────────────────────────────────────────────────────

function openExportModal() {
  exportModal.classList.remove('hidden');
  exportConfirmBtn.focus();
}

async function confirmExport() {
  const format = document.querySelector('input[name="export-fmt"]:checked')?.value ?? 'json';
  exportModal.classList.add('hidden');
  clearError();
  try {
    const content = await invoke('export_items', { dir: storeDir, format });
    const ext = format === 'toon' ? 'toon' : format;
    const savePath = await invoke('plugin:dialog|save', {
      options: { defaultPath: `crumbs_export.${ext}`, title: 'Save export' },
    });
    if (!savePath) return;
    await invoke('write_text_file', { path: savePath, content });
  } catch (e) {
    showError(`Export failed: ${e}`);
  }
}

exportBtn.addEventListener('click', openExportModal);
exportCancelBtn.addEventListener('click', () => { exportModal.classList.add('hidden'); });
exportConfirmBtn.addEventListener('click', confirmExport);
exportModal.addEventListener('keydown', e => {
  if (e.key === 'Enter') confirmExport();
  if (e.key === 'Escape') { exportModal.classList.add('hidden'); e.stopPropagation(); }
});
exportModal.addEventListener('click', e => {
  if (e.target === exportModal) exportModal.classList.add('hidden');
});

// ── Reindex ───────────────────────────────────────────────────────────────

reindexBtn.addEventListener('click', async () => {
  clearError();
  try {
    await invoke('reindex_store', { dir: storeDir });
    await loadItems();
  } catch (e) {
    showError(`Reindex failed: ${e}`);
  }
});

// ── Delete modal ──────────────────────────────────────────────────────────

function openDeleteModal() {
  deleteModal.classList.remove('hidden');
  deleteConfirmBtn.focus();
}

async function confirmDelete() {
  deleteModal.classList.add('hidden');
  clearError();
  try {
    await invoke('delete_item', { dir: storeDir, id: selectedId });
    selectedId = null;
    await loadItems();
  } catch (e) {
    showError(`Delete failed: ${e}`);
  }
}

// ── Close modal ───────────────────────────────────────────────────────────

function openCloseModal(id) {
  pendingCloseId = id;
  closeReason.value = '';
  closeModal.classList.remove('hidden');
  closeReason.focus();
}

async function confirmClose() {
  closeModal.classList.add('hidden');
  clearError();
  try {
    await invoke('close_item', { dir: storeDir, id: pendingCloseId, reason: closeReason.value.trim() });
    if (selectedId === pendingCloseId && !showClosedEl.checked) {
      selectedId = null;
    }
    await loadItems();
  } catch (e) {
    showError(`Close failed: ${e}`);
  }
  pendingCloseId = '';
}

// ── Blocked-by modal ──────────────────────────────────────────────────────

function renderBlockerList(filterText) {
  const item = selectedItem();
  if (!item) return;
  const currentBlockers = item.blocked_by ?? [];
  const candidates = allItems.filter(i =>
    i.id !== selectedId &&
    i.status !== 'closed' &&
    (!filterText || i.title.toLowerCase().includes(filterText.toLowerCase()) ||
      i.id.toLowerCase().includes(filterText.toLowerCase()))
  );
  blockerList.innerHTML = candidates.map(i => `
    <label>
      <input type="checkbox" data-id="${escHtml(i.id)}"${currentBlockers.includes(i.id) ? ' checked' : ''}>
      <span class="item-id">${escHtml(i.id)}</span> ${escHtml(i.title)}
    </label>
  `).join('');
}

function openBlockedByModal() {
  const item = selectedItem();
  if (!item) return;
  blockerTargetTitle.textContent = item.title;
  blockerSearch.value = '';
  renderBlockerList('');
  blockedByModal.classList.remove('hidden');
  blockerSearch.focus();
}

async function confirmBlockedBy() {
  blockedByModal.classList.add('hidden');
  const item = selectedItem();
  if (!item) return;

  const oldBlockers = item.blocked_by ?? [];
  const checked = [...blockerList.querySelectorAll('input[type="checkbox"]')];
  const newBlockers = checked.filter(cb => cb.checked).map(cb => cb.dataset.id);

  const toAdd    = newBlockers.filter(id => !oldBlockers.includes(id));
  const toRemove = oldBlockers.filter(id => !newBlockers.includes(id));

  clearError();
  try {
    if (toAdd.length > 0) {
      await invoke('link_items', { dir: storeDir, id: selectedId, relation: 'blocked-by', targets: toAdd, remove: false });
    }
    if (toRemove.length > 0) {
      await invoke('link_items', { dir: storeDir, id: selectedId, relation: 'blocked-by', targets: toRemove, remove: true });
    }
    await loadItems();
  } catch (e) {
    showError(`Block failed: ${e}`);
  }
}

async function createNewBlocker() {
  const title = newBlockerTitle.value.trim();
  if (!title) return;
  clearError();
  try {
    await invoke('create_item', { dir: storeDir, title });
    newBlockerTitle.value = '';
    // Reload so the new item appears in the list, pre-checked.
    allItems = await invoke('list_items', { dir: storeDir, includeClosed: showClosedEl.checked });
    // Find the newly created item by title (most recently created match).
    const newItem = allItems
      .filter(i => i.title === title)
      .sort((a, b) => b.created.localeCompare(a.created))[0];
    renderBlockerList(blockerSearch.value);
    if (newItem) {
      const cb = blockerList.querySelector(`input[data-id="${CSS.escape(newItem.id)}"]`);
      if (cb) cb.checked = true;
    }
  } catch (e) {
    showError(`Create failed: ${e}`);
  }
}

newBlockerBtn.addEventListener('click', createNewBlocker);
newBlockerTitle.addEventListener('keydown', e => {
  if (e.key === 'Enter') { e.preventDefault(); createNewBlocker(); }
});

blockerSearch.addEventListener('input', () => renderBlockerList(blockerSearch.value));
blockerCancelBtn.addEventListener('click', () => { blockedByModal.classList.add('hidden'); });
blockerConfirmBtn.addEventListener('click', confirmBlockedBy);
blockedByModal.addEventListener('keydown', e => {
  if (e.key === 'Escape') { blockedByModal.classList.add('hidden'); e.stopPropagation(); }
});
blockedByModal.addEventListener('click', e => {
  if (e.target === blockedByModal) blockedByModal.classList.add('hidden');
});

// ── Defer modal ───────────────────────────────────────────────────────────

function openDeferModal() {
  deferUntil.value = '';
  deferModal.classList.remove('hidden');
  deferUntil.focus();
}

async function confirmDefer() {
  deferModal.classList.add('hidden');
  if (!selectedId) return;
  clearError();
  try {
    await invoke('defer_item', { dir: storeDir, id: selectedId, until: deferUntil.value });
    await loadItems();
  } catch (e) {
    showError(`Defer failed: ${e}`);
  }
}

deferCancelBtn.addEventListener('click', () => { deferModal.classList.add('hidden'); });
deferConfirmBtn.addEventListener('click', confirmDefer);
deferModal.addEventListener('keydown', e => {
  if (e.key === 'Enter') confirmDefer();
  if (e.key === 'Escape') { deferModal.classList.add('hidden'); e.stopPropagation(); }
});
deferModal.addEventListener('click', e => {
  if (e.target === deferModal) deferModal.classList.add('hidden');
});

// ── Timer modal ───────────────────────────────────────────────────────────

function openTimerModal() {
  if (!selectedId) return;
  const item = selectedItem();
  const starting = !hasActiveTimer(item?.description);
  timerModalTitle.textContent = starting ? 'Start timer' : 'Stop timer';
  timerConfirmBtn.textContent = starting ? 'Start' : 'Stop';
  timerComment.value = '';
  timerModal.classList.remove('hidden');
  timerComment.focus();
}

async function confirmTimer() {
  timerModal.classList.add('hidden');
  if (!selectedId) return;
  const item = selectedItem();
  const starting = !hasActiveTimer(item?.description);
  const comment = timerComment.value.trim();
  clearError();
  try {
    if (starting) {
      await invoke('start_timer', { dir: storeDir, id: selectedId, comment });
    } else {
      await invoke('stop_timer', { dir: storeDir, id: selectedId, comment });
    }
    await loadItems();
  } catch (e) {
    showError(`Timer failed: ${e}`);
  }
}

timerCancelBtn.addEventListener('click', () => { timerModal.classList.add('hidden'); });
timerConfirmBtn.addEventListener('click', confirmTimer);
timerModal.addEventListener('keydown', e => {
  if (e.key === 'Enter') confirmTimer();
  if (e.key === 'Escape') { timerModal.classList.add('hidden'); e.stopPropagation(); }
});
timerModal.addEventListener('click', e => {
  if (e.target === timerModal) timerModal.classList.add('hidden');
});

// ── New item modal ────────────────────────────────────────────────────────

function openNewModal() {
  newTitle.value = '';
  newModal.classList.remove('hidden');
  newTitle.focus();
}

async function confirmNew() {
  const title = newTitle.value.trim();
  if (!title) return;
  newModal.classList.add('hidden');
  clearError();
  try {
    await invoke('create_item', { dir: storeDir, title });
    await loadItems();
    // Newly created items are always `open` so they appear regardless of showClosed.
    const newItem = allItems
      .filter(i => i.title === title)
      .sort((a, b) => b.created.localeCompare(a.created))[0];
    if (newItem) {
      selectRow(newItem.id);
      view.focus();
    }
  } catch (e) {
    showError(`Create failed: ${e}`);
  }
}

// ── Open Dir modal ────────────────────────────────────────────────────────

function openOpenDirModal() {
  dirPathInput.value = storeDir ? storeDir.replace(/\/\.crumbs$/, '') : '';
  initPrompt.classList.add('hidden');
  openDirActions.classList.remove('hidden');
  openDirModal.classList.remove('hidden');
  dirPathInput.focus();
}

async function checkAndOpenDir(rawPath) {
  const path = rawPath.trim();
  if (!path) return;

  const hasStore = await invoke('has_store', { dir: path });
  if (hasStore) {
    // Prefer the .crumbs subdirectory (project store); fall back to path itself (global store).
    const hasSubStore = await invoke('has_store', { dir: `${path}/.crumbs` });
    await switchStore(hasSubStore ? `${path}/.crumbs` : path);
    openDirModal.classList.add('hidden');
  } else {
    initPrompt.classList.remove('hidden');
    openDirActions.classList.add('hidden');
    dirPathInput.value = path;
  }
}

async function switchStore(crumbsDir) {
  storeDir = crumbsDir;
  storePathEl.textContent = storeDir;
  selectedId = null;
  searchResults = null;
  searchInput.value = '';
  addRecentStore(storeDir);
  renderSidebar();
  await loadItems();
}

// ── Recent stores (localStorage) ─────────────────────────────────────────

function getRecentStores() {
  return JSON.parse(localStorage.getItem('crumbs_recent_stores') || '[]');
}

function addRecentStore(path) {
  let stores = getRecentStores();
  if (!stores.includes(path)) {
    stores.push(path);
    stores.sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()));
    if (stores.length > 20) stores = stores.slice(0, 20);
    localStorage.setItem('crumbs_recent_stores', JSON.stringify(stores));
  }
}

function removeRecentStore(path) {
  const stores = getRecentStores().filter(p => p !== path);
  localStorage.setItem('crumbs_recent_stores', JSON.stringify(stores));
}

function getStoreAliases() {
  return JSON.parse(localStorage.getItem('crumbs_store_aliases') || '{}');
}
function setStoreAlias(path, name) {
  const aliases = getStoreAliases();
  if (name.trim()) {
    aliases[path] = name.trim();
  } else {
    delete aliases[path];
  }
  localStorage.setItem('crumbs_store_aliases', JSON.stringify(aliases));
}
function storeDisplayName(path) {
  const aliases = getStoreAliases();
  return aliases[path] || storeBaseName(path);
}

function storeBaseName(crumbsPath) {
  // Always show "parent/project" — last 2 meaningful components after stripping .crumbs.
  // "/client-a/app/.crumbs" → "client-a/app"
  // "/Application Support/crumbs" → "Application Support/crumbs"
  const normalized = crumbsPath.replace(/\/\.crumbs$/, '').replace(/\/$/, '');
  const parts = normalized.split('/').filter(Boolean);
  if (parts.length >= 2) {
    return `${parts[parts.length - 2]}/${parts[parts.length - 1]}`;
  }
  return parts[parts.length - 1] || crumbsPath;
}

function renderSidebar() {
  const stores = getRecentStores().slice().sort((a, b) =>
    storeDisplayName(a).toLowerCase().localeCompare(storeDisplayName(b).toLowerCase())
  );
  storeListEl.innerHTML = stores.map(p => `
    <div class="store-item${p === storeDir ? ' active' : ''}" data-path="${escHtml(p)}">
      <span class="store-name" title="Double-click to rename">${escHtml(storeDisplayName(p))}</span>
      <span class="store-item-path">${escHtml(p)}</span>
    </div>
  `).join('');

}

// ── Row → sidebar drag (pointer events; bypasses WKWebView HTML5 DnD bugs) ─
// HTML5 drag-and-drop is unreliable in Tauri's WKWebView: dragover may not
// fire and dataTransfer.getData() often returns empty on drop. We simulate
// drag using mousedown/mousemove/mouseup + elementFromPoint for hit-testing.

let dragGhost = null;

function clearDropTargets() {
  for (const el of storeListEl.querySelectorAll('.drop-target')) {
    el.classList.remove('drop-target');
  }
}

function sidebarTargetAt(x, y) {
  // Ghost has pointer-events:none so elementFromPoint sees through it.
  const el = document.elementFromPoint(x, y);
  const item = el?.closest('.store-item[data-path]');
  return item && item.dataset.path !== storeDir ? item : null;
}

function startRowDrag(e, item, tr) {
  if (e.button !== 0) return;            // left-click only
  e.preventDefault();                    // prevent text selection

  dragItemId = item.id;
  tr.classList.add('dragging');

  // Floating ghost label
  dragGhost = document.createElement('div');
  dragGhost.className = 'drag-ghost';
  dragGhost.textContent = item.title;
  dragGhost.style.left = `${e.clientX + 14}px`;
  dragGhost.style.top  = `${e.clientY - 10}px`;
  document.body.appendChild(dragGhost);

  function onMove(ev) {
    dragGhost.style.left = `${ev.clientX + 14}px`;
    dragGhost.style.top  = `${ev.clientY - 10}px`;
    const target = sidebarTargetAt(ev.clientX, ev.clientY);
    for (const el of storeListEl.querySelectorAll('.store-item')) {
      el.classList.toggle('drop-target', el === target);
    }
  }

  async function onUp(ev) {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
    dragGhost.remove(); dragGhost = null;
    tr.classList.remove('dragging');
    clearDropTargets();

    const target = sidebarTargetAt(ev.clientX, ev.clientY);
    const id = dragItemId;
    dragItemId = null;
    if (!target || !id) return;

    clearError();
    try {
      await invoke('move_item', { srcDir: storeDir, id, dstDir: target.dataset.path });
      if (selectedId === id) selectedId = null;
      await loadItems();
    } catch (err) {
      showError(`Move failed: ${err}`);
    }
  }

  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
}

storeListEl.addEventListener('click', async e => {
  const item = e.target.closest('.store-item[data-path]');
  if (!item) return;
  if (e.target.classList.contains('store-name-input')) return;
  const path = item.dataset.path;
  if (path !== storeDir) await switchStore(path);
});

function beginRename(item) {
  const nameSpan = item.querySelector('.store-name');
  if (!nameSpan || item.querySelector('.store-name-input')) return;
  const path = item.dataset.path;
  const current = storeDisplayName(path);
  const input = document.createElement('input');
  input.type = 'text';
  input.className = 'store-name-input';
  input.value = current;
  nameSpan.replaceWith(input);
  input.focus();
  input.select();
  function commit() {
    setStoreAlias(path, input.value);
    renderSidebar();
  }
  input.addEventListener('blur', commit);
  input.addEventListener('keydown', e => {
    if (e.key === 'Enter') { e.preventDefault(); input.blur(); }
    if (e.key === 'Escape') { input.removeEventListener('blur', commit); renderSidebar(); e.stopPropagation(); }
  });
}

storeListEl.addEventListener('dblclick', e => {
  const item = e.target.closest('.store-item[data-path]');
  if (!item) return;
  beginRename(item);
});

// Context menu
const storeContextMenu = document.getElementById('store-context-menu');
const ctxRenameBtn = document.getElementById('ctx-rename-btn');
let ctxTargetPath = null;

function hideContextMenu() {
  storeContextMenu.classList.add('hidden');
}

storeListEl.addEventListener('contextmenu', e => {
  const item = e.target.closest('.store-item[data-path]');
  if (!item) return;
  e.preventDefault();
  ctxTargetPath = item.dataset.path;
  storeContextMenu.style.left = `${e.clientX}px`;
  storeContextMenu.style.top = `${e.clientY}px`;
  storeContextMenu.classList.remove('hidden');
});

ctxRenameBtn.addEventListener('click', () => {
  const path = ctxTargetPath;
  hideContextMenu();
  if (!path) return;
  const item = storeListEl.querySelector(`.store-item[data-path="${CSS.escape(path)}"]`);
  if (item) beginRename(item);
});

document.addEventListener('click', e => {
  if (!storeContextMenu.contains(e.target)) hideContextMenu();
});
document.addEventListener('keydown', e => {
  const mod = e.metaKey || e.ctrlKey;

  // Escape — dismiss help modal or context menu
  if (e.key === 'Escape') {
    hideContextMenu();
    if (!helpModal.classList.contains('hidden')) {
      helpModal.classList.add('hidden');
    }
    return;
  }

  // ? — open help modal
  if (e.key === '?' && !isControlFocused() && !isModalOpen()) {
    helpModal.classList.remove('hidden');
    return;
  }

  // Cmd/Ctrl+N — new item
  if (mod && e.key === 'n' && !isControlFocused() && !isModalOpen()) {
    e.preventDefault();
    openNewModal();
    return;
  }

  // Cmd/Ctrl+F — focus search bar (when no interactive control is focused)
  if (mod && e.key === 'f' && !isControlFocused() && !isModalOpen()) {
    e.preventDefault();
    searchInput.focus();
    searchInput.select();
    return;
  }

  // Cmd/Ctrl+R — always prevent native webview reload; only run loadItems when no modal is open
  if (mod && e.key === 'r') {
    e.preventDefault();
    if (!isModalOpen()) loadItems();
    return;
  }

  // Navigation and selection shortcuts — suppressed when any input/editor focused or modal open
  if (isControlFocused() || isModalOpen()) return;

  // ↑ / ↓ — row navigation
  if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
    const rows = [...document.querySelectorAll('#items-body tr[data-id]')];
    if (!rows.length) return;
    e.preventDefault();
    const currentIndex = rows.findIndex(r => r.dataset.id === selectedId);
    let nextIndex;
    if (e.key === 'ArrowUp') {
      nextIndex = currentIndex <= 0 ? 0 : currentIndex - 1;
    } else {
      nextIndex = currentIndex >= rows.length - 1 ? rows.length - 1 : currentIndex + 1;
    }
    selectRow(rows[nextIndex].dataset.id, rows[nextIndex]);
    return;
  }

  // Enter — focus CM6 editor for selected item
  if (e.key === 'Enter' && selectedId) {
    if (!detailPane.classList.contains('hidden')) {
      view.focus();
    }
    return;
  }

  // Delete/Backspace — open delete modal for selected item
  if ((e.key === 'Delete' || e.key === 'Backspace') && selectedId) {
    e.preventDefault();
    openDeleteModal();
    return;
  }
});

sidebarAddBtn.addEventListener('click', openOpenDirModal);

sidebarRemoveBtn.addEventListener('click', () => {
  if (!storeDir) return;
  removeRecentStore(storeDir);
  renderSidebar();
});

// ── Sidebar resizer ───────────────────────────────────────────────────────

(function initSidebarResizer() {
  let dragging = false;
  let startX = 0;
  let startW = 0;

  sidebarResizer.addEventListener('mousedown', e => {
    dragging = true;
    startX = e.clientX;
    startW = sidebarEl.getBoundingClientRect().width;
    sidebarResizer.classList.add('resizing');
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'col-resize';
  });

  document.addEventListener('mousemove', e => {
    if (!dragging) return;
    const w = Math.max(140, Math.min(360, startW + e.clientX - startX));
    sidebarEl.style.width = `${w}px`;
  });

  document.addEventListener('mouseup', () => {
    if (!dragging) return;
    dragging = false;
    sidebarResizer.classList.remove('resizing');
    document.body.style.userSelect = '';
    document.body.style.cursor = '';
  });
})();

// ── Detail pane left/right resizer ────────────────────────────────────────

(function initDetailResizer() {
  let dragging = false;
  let startX = 0;
  let startW = 0;

  detailResizer.addEventListener('mousedown', e => {
    dragging = true;
    startX = e.clientX;
    startW = detailLeft.getBoundingClientRect().width;
    detailResizer.classList.add('resizing');
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'col-resize';
  });

  document.addEventListener('mousemove', e => {
    if (!dragging) return;
    const w = Math.max(140, Math.min(480, startW + e.clientX - startX));
    detailLeft.style.width = `${w}px`;
  });

  document.addEventListener('mouseup', () => {
    if (!dragging) return;
    dragging = false;
    detailResizer.classList.remove('resizing');
    document.body.style.userSelect = '';
    document.body.style.cursor = '';
  });
})();

// ── Sidebar toggle ────────────────────────────────────────────────────────

(function initSidebarToggle() {
  const hidden = localStorage.getItem('sidebar_hidden') === 'true';
  if (hidden) {
    sidebarEl.classList.add('hidden');
    sidebarResizer.classList.add('hidden');
  }
  sidebarToggleBtn.addEventListener('click', () => {
    const nowHidden = sidebarEl.classList.toggle('hidden');
    sidebarResizer.classList.toggle('hidden', nowHidden);
    localStorage.setItem('sidebar_hidden', nowHidden);
  });
})();

// ── Theme toggle ──────────────────────────────────────────────────────────

(function initTheme() {
  const stored = localStorage.getItem('theme');
  const prefersDark = globalThis.matchMedia('(prefers-color-scheme: dark)').matches;
  const initial = stored ?? (prefersDark ? 'dark' : 'light');
  document.documentElement.dataset.theme = initial;
})();

themeBtn.addEventListener('click', () => {
  const next = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
  document.documentElement.dataset.theme = next;
  localStorage.setItem('theme', next);
});

// ── Event wiring ──────────────────────────────────────────────────────────

refreshBtn.addEventListener('click', loadItems);
showClosedEl.addEventListener('change', loadItems);

// Status filter buttons
for (const btn of document.querySelectorAll('.filter-btn')) {
  btn.addEventListener('click', async () => {
    for (const b of document.querySelectorAll('.filter-btn')) b.classList.remove('active');
    btn.classList.add('active');
    filterStatus = btn.dataset.status;
    if (filterStatus === 'closed' && !showClosedEl.checked) {
      showClosedEl.checked = true;
      await loadItems();
    } else {
      renderTable();
    }
  });
}

// Additional filter controls
document.getElementById('filter-priority').addEventListener('change', e => { filterPriority = e.target.value; renderTable(); });
document.getElementById('filter-type').addEventListener('change', e => { filterType = e.target.value; renderTable(); });
document.getElementById('filter-tag').addEventListener('input', e => { filterTag = e.target.value.trim(); renderTable(); });

// Column picker
const colPickerBtn  = document.getElementById('col-picker-btn');
const colPickerMenu = document.getElementById('col-picker-menu');

function renderColPicker() {
  colPickerMenu.innerHTML = ALL_COLUMNS.map(col => `
    <label>
      <input type="checkbox" data-col="${col.key}" ${visibleCols.includes(col.key) ? 'checked' : ''}>
      ${escHtml(col.label)}
    </label>
  `).join('');
  colPickerMenu.querySelectorAll('input[type=checkbox]').forEach(cb => {
    cb.addEventListener('change', () => {
      visibleCols = ALL_COLUMNS.map(c => c.key).filter(key => {
        const el = colPickerMenu.querySelector(`input[data-col="${key}"]`);
        return el ? el.checked : false;
      });
      saveVisibleColumns(visibleCols);
      rebuildTableHeader();
      initColResizers();
      renderTable();
    });
  });
}

colPickerBtn.addEventListener('click', e => {
  e.stopPropagation();
  renderColPicker();
  const rect = colPickerBtn.getBoundingClientRect();
  colPickerMenu.style.top  = `${rect.bottom + 4}px`;
  colPickerMenu.style.left = `${rect.left}px`;
  colPickerMenu.classList.toggle('hidden');
});
document.addEventListener('click', e => {
  if (!colPickerMenu.contains(e.target) && e.target !== colPickerBtn) {
    colPickerMenu.classList.add('hidden');
  }
});

// Row click → select item
itemsBody.addEventListener('click', e => {
  const tr = e.target.closest('tr[data-id]');
  if (!tr) return;
  selectRow(tr.dataset.id, tr);
});

// Inline rename via double-click on detail pane title
detailTitleLabel.addEventListener('dblclick', () => {
  const id = detailTitleLabel.dataset.id;
  if (!id) return;
  const current = detailTitleLabel.textContent;
  const input = document.createElement('input');
  input.type = 'text';
  input.value = current;
  input.style.cssText = 'width:100%;font:inherit;background:var(--bg);color:var(--text);border:1px solid var(--accent);border-radius:3px;padding:0 4px;outline:none;box-sizing:border-box;';
  detailTitleLabel.replaceWith(input);
  input.focus();
  input.select();
  function commit() {
    const newTitle = input.value.trim();
    if (newTitle && newTitle !== current) doUpdateTitle(id, newTitle);
    input.replaceWith(detailTitleLabel);
  }
  input.addEventListener('blur', commit);
  input.addEventListener('keydown', e => {
    if (e.key === 'Enter') { e.preventDefault(); input.blur(); }
    if (e.key === 'Escape') { input.removeEventListener('blur', commit); input.replaceWith(detailTitleLabel); e.stopPropagation(); }
  });
});

// Body text autosave: debounced on input (2s), immediate on blur or Cmd/Ctrl-S
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
// Input, blur, and Cmd+S are handled by the CM6 updateListener, domEventHandlers, and keymap.

previewBtn.addEventListener('click', () => setPreviewMode(!previewMode));

// ── Emoji picker ──────────────────────────────────────────────────────────────

const EMOJI_DATA = [
  { cat: '😀', label: 'Smileys', emoji: [
    ['+1','👍'],['smile','😄'],['laughing','😆'],['joy','😂'],['rofl','🤣'],
    ['blush','😊'],['slightly_smiling_face','🙂'],['wink','😉'],['heart_eyes','😍'],
    ['kissing_heart','😘'],['stuck_out_tongue','😛'],['stuck_out_tongue_winking_eye','😜'],
    ['thinking','🤔'],['zipper_mouth_face','🤐'],['raised_eyebrow','🤨'],
    ['neutral_face','😐'],['expressionless','😑'],['smirk','😏'],['unamused','😒'],
    ['roll_eyes','🙄'],['grimacing','😬'],['lying_face','🤥'],['relieved','😌'],
    ['pensive','😔'],['sleepy','😪'],['drooling_face','🤤'],['sleeping','😴'],
    ['mask','😷'],['face_with_thermometer','🤒'],['nauseated_face','🤢'],
    ['sneezing_face','🤧'],['hot_face','🥵'],['cold_face','🥶'],['woozy_face','🥴'],
    ['dizzy_face','😵'],['exploding_head','🤯'],['cowboy_hat_face','🤠'],
    ['partying_face','🥳'],['sunglasses','😎'],['nerd_face','🤓'],['monocle_face','🧐'],
    ['confused','😕'],['worried','😟'],['slightly_frowning_face','🙁'],
    ['frowning_face','☹️'],['open_mouth','😮'],['hushed','😯'],['astonished','😲'],
    ['flushed','😳'],['pleading_face','🥺'],['anguished','😧'],['fearful','😨'],
    ['cold_sweat','😰'],['disappointed_relieved','😥'],['cry','😢'],['sob','😭'],
    ['scream','😱'],['confounded','😖'],['persevere','😣'],['disappointed','😞'],
    ['sweat','😓'],['weary','😩'],['tired_face','😫'],['yawning_face','🥱'],
    ['triumph','😤'],['rage','😡'],['angry','😠'],['skull','💀'],
    ['poop','💩'],['clown_face','🤡'],['japanese_ogre','👹'],['japanese_goblin','👺'],
    ['ghost','👻'],['alien','👽'],['space_invader','👾'],['robot','🤖'],
  ]},
  { cat: '👋', label: 'People', emoji: [
    ['wave','👋'],['raised_hand','✋'],['ok_hand','👌'],['v','✌️'],['crossed_fingers','🤞'],
    ['metal','🤘'],['point_up_2','👆'],['point_down','👇'],['point_left','👈'],
    ['point_right','👉'],['fu','🖕'],['raised_hands','🙌'],['clap','👏'],
    ['handshake','🤝'],['pray','🙏'],['writing_hand','✍️'],['nail_care','💅'],
    ['ear','👂'],['nose','👃'],['eyes','👀'],['eye','👁️'],['tongue','👅'],
    ['lips','👄'],['baby','👶'],['boy','👦'],['girl','👧'],['man','👨'],['woman','👩'],
    ['man_with_blond_hair','👱'],['man_in_tuxedo','🤵'],['bride_with_veil','👰'],
    ['pregnant_woman','🤰'],['person_fencing','🤺'],['horse_racing','🏇'],
    ['snowboarder','🏂'],['surfer','🏄'],['rowboat','🚣'],['swimmer','🏊'],
    ['bicyclist','🚴'],['busts_in_silhouette','👥'],['walking','🚶'],
    ['runner','🏃'],['dancer','💃'],['man_dancing','🕺'],
    ['family','👪'],['couple','👫'],['two_women_holding_hands','👭'],
    ['two_men_holding_hands','👬'],['couplekiss','💏'],['couple_with_heart','💑'],
    ['cop','👮'],['construction_worker','👷'],['guardsman','💂'],
    ['sleuth_or_spy','🕵️'],['santa','🎅'],['angel','👼'],
    ['princess','👸'],['prince','🤴'],['older_woman','👵'],['older_man','👴'],
    ['man_with_turban','👳'],['man_with_gua_pi_mao','👲'],
  ]},
  { cat: '🐶', label: 'Animals', emoji: [
    ['dog','🐶'],['cat','🐱'],['mouse','🐭'],['hamster','🐹'],['rabbit','🐰'],
    ['fox_face','🦊'],['bear','🐻'],['panda_face','🐼'],['koala','🐨'],['tiger','🐯'],
    ['lion','🦁'],['cow','🐮'],['pig','🐷'],['frog','🐸'],['monkey','🐵'],
    ['chicken','🐔'],['penguin','🐧'],['bird','🐦'],['baby_chick','🐤'],
    ['hatching_chick','🐣'],['duck','🦆'],['eagle','🦅'],['owl','🦉'],['bat','🦇'],
    ['wolf','🐺'],['boar','🐗'],['horse','🐴'],['unicorn','🦄'],['bee','🐝'],
    ['bug','🐛'],['butterfly','🦋'],['snail','🐌'],['shell','🐚'],['beetle','🐞'],
    ['ant','🐜'],['mosquito','🦟'],['cricket','🦗'],['spider','🕷️'],['scorpion','🦂'],
    ['turtle','🐢'],['snake','🐍'],['lizard','🦎'],['dragon_face','🐲'],['dragon','🐉'],
    ['sauropod','🦕'],['t-rex','🦖'],['whale','🐳'],['whale2','🐋'],['dolphin','🐬'],
    ['fish','🐟'],['tropical_fish','🐠'],['blowfish','🐡'],['shark','🦈'],
    ['octopus','🐙'],['crab','🦀'],['lobster','🦞'],['shrimp','🦐'],['squid','🦑'],
  ]},
  { cat: '🍎', label: 'Food', emoji: [
    ['apple','🍎'],['green_apple','🍏'],['pear','🍐'],['tangerine','🍊'],['lemon','🍋'],
    ['banana','🍌'],['watermelon','🍉'],['grapes','🍇'],['strawberry','🍓'],
    ['melon','🍈'],['cherries','🍒'],['peach','🍑'],['mango','🥭'],['pineapple','🍍'],
    ['coconut','🥥'],['kiwi_fruit','🥝'],['tomato','🍅'],['eggplant','🍆'],
    ['avocado','🥑'],['broccoli','🥦'],['leafy_green','🥬'],['cucumber','🥒'],
    ['hot_pepper','🌶️'],['corn','🌽'],['carrot','🥕'],['garlic','🧄'],['onion','🧅'],
    ['potato','🥔'],['sweet_potato','🍠'],['croissant','🥐'],['bagel','🥯'],
    ['bread','🍞'],['baguette_bread','🥖'],['pretzel','🥨'],['cheese','🧀'],['egg','🥚'],
    ['cooking','🍳'],['pancakes','🥞'],['waffle','🧇'],['bacon','🥓'],['cut_of_meat','🥩'],
    ['poultry_leg','🍗'],['meat_on_bone','🍖'],['hotdog','🌭'],['hamburger','🍔'],
    ['fries','🍟'],['pizza','🍕'],['sandwich','🥪'],['stuffed_flatbread','🥙'],
    ['taco','🌮'],['burrito','🌯'],['salad','🥗'],['shallow_pan_of_food','🥘'],
    ['spaghetti','🍝'],['ramen','🍜'],['stew','🍲'],['curry','🍛'],['sushi','🍣'],
    ['bento','🍱'],['dumpling','🥟'],['fried_shrimp','🍤'],['rice_ball','🍙'],
    ['rice','🍚'],['rice_cracker','🍘'],['fish_cake','🍥'],['fortune_cookie','🥠'],
    ['moon_cake','🥮'],['oden','🍢'],['dango','🍡'],['shaved_ice','🍧'],
    ['ice_cream','🍨'],['icecream','🍦'],['pie','🥧'],['cake','🎂'],['birthday','🎂'],
    ['shortcake','🍰'],['cupcake','🧁'],['candy','🍬'],['lollipop','🍭'],
    ['chocolate_bar','🍫'],['popcorn','🍿'],['doughnut','🍩'],['cookie','🍪'],
    ['honey_pot','🍯'],['salt','🧂'],['coffee','☕'],['tea','🍵'],['boba','🧋'],
    ['beer','🍺'],['beers','🍻'],['champagne','🍾'],['wine_glass','🍷'],
    ['cocktail','🍸'],['tropical_drink','🍹'],['beverage_box','🧃'],
    ['milk_glass','🥛'],['cup_with_straw','🥤'],
  ]},
  { cat: '✈️', label: 'Travel', emoji: [
    ['car','🚗'],['taxi','🚕'],['bus','🚌'],['trolleybus','🚎'],['racing_car','🏎️'],
    ['police_car','🚓'],['ambulance','🚑'],['fire_engine','🚒'],['minibus','🚐'],
    ['truck','🚚'],['articulated_lorry','🚛'],['tractor','🚜'],['kick_scooter','🛴'],
    ['bike','🚲'],['motor_scooter','🛵'],['motorcycle','🏍️'],['monorail','🚝'],
    ['mountain_railway','🚞'],['train','🚋'],['train2','🚆'],['bullettrain_side','🚄'],
    ['bullettrain_front','🚅'],['light_rail','🚈'],['steam_locomotive','🚂'],
    ['railway_car','🚃'],['station','🚉'],['airplane','✈️'],['small_airplane','🛩️'],
    ['flight_departure','🛫'],['flight_arrival','🛬'],['seat','💺'],['helicopter','🚁'],
    ['suspension_railway','🚟'],['mountain_cableway','🚠'],['aerial_tramway','🚡'],
    ['rocket','🚀'],['flying_saucer','🛸'],['boat','⛵'],['sailboat','⛵'],
    ['canoe','🛶'],['speedboat','🚤'],['ship','🚢'],['ferry','⛴️'],
    ['anchor','⚓'],['construction','🚧'],['fuelpump','⛽'],['busstop','🚏'],
    ['vertical_traffic_light','🚦'],['traffic_light','🚥'],['rotating_light','🚨'],
    ['passport_control','🛂'],['customs','🛃'],['baggage_claim','🛄'],
    ['left_luggage','🛅'],['moyai','🗿'],['statue_of_liberty','🗽'],
    ['tokyo_tower','🗼'],['european_castle','🏰'],['japanese_castle','🏯'],
    ['stadium','🏟️'],['ferris_wheel','🎡'],['roller_coaster','🎢'],['carousel_horse','🎠'],
    ['fountain','⛲'],['camping','🏕️'],['beach_umbrella','🏖️'],['desert_island','🏝️'],
    ['national_park','🏞️'],['sunrise','🌅'],['sunrise_over_mountains','🌄'],
    ['city_sunrise','🌇'],['city_sunset','🌆'],['cityscape_at_dusk','🌆'],
    ['night_with_stars','🌃'],['milky_way','🌌'],['bridge_at_night','🌉'],
    ['foggy','🌁'],
  ]},
  { cat: '💡', label: 'Objects', emoji: [
    ['watch','⌚'],['iphone','📱'],['calling','📲'],['computer','💻'],
    ['keyboard','⌨️'],['desktop_computer','🖥️'],['printer','🖨️'],['mouse_three_button','🖱️'],
    ['trackball','🖲️'],['minidisc','💽'],['floppy_disk','💾'],['cd','💿'],['dvd','📀'],
    ['abacus','🧮'],['movie_camera','🎥'],['film_strip','🎞️'],['film_projector','📽️'],
    ['clapper','🎬'],['tv','📺'],['camera','📷'],['camera_flash','📸'],
    ['video_camera','📹'],['vhs','📼'],['bulb','💡'],['flashlight','🔦'],
    ['candle','🕯️'],['wastebasket','🗑️'],['oil_drum','🛢️'],['money_with_wings','💸'],
    ['dollar','💵'],['euro','💶'],['pound','💷'],['yen','💴'],['credit_card','💳'],
    ['gem','💎'],['chart','💹'],['briefcase','💼'],['file_folder','📁'],
    ['open_file_folder','📂'],['card_index_dividers','🗂️'],['newspaper_roll','🗞️'],
    ['newspaper','📰'],['notebook','📓'],['notebook_with_decorative_cover','📔'],
    ['ledger','📒'],['books','📚'],['book','📖'],['link','🔗'],['paperclip','📎'],
    ['paperclips','🖇️'],['scissors','✂️'],['triangular_ruler','📐'],['straight_ruler','📏'],
    ['lock','🔒'],['unlock','🔓'],['key','🔑'],['old_key','🗝️'],['hammer','🔨'],
    ['axe','🪓'],['pick','⛏️'],['hammer_and_pick','⚒️'],['hammer_and_wrench','🛠️'],
    ['dagger','🗡️'],['sword','⚔️'],['gun','🔫'],['bow_and_arrow','🏹'],['shield','🛡️'],
    ['wrench','🔧'],['nut_and_bolt','🔩'],['gear','⚙️'],['compression','🗜️'],
    ['scales','⚖️'],['probing_cane','🦯'],['link','🔗'],['chains','⛓️'],['hook','🪝'],
    ['toolbox','🧰'],['magnet','🧲'],['ladder','🪜'],['stethoscope','🩺'],
    ['syringe','💉'],['pill','💊'],['bandage','🩹'],['adhesive_bandage','🩹'],
    ['door','🚪'],['bed','🛏️'],['couch_and_lamp','🛋️'],['toilet','🚽'],
    ['shower','🚿'],['bathtub','🛁'],['shopping_cart','🛒'],
    ['smoking','🚬'],['coffin','⚰️'],['urn','⚱️'],['amphora','🏺'],
    ['crystal_ball','🔮'],['compass','🧭'],['teddy_bear','🧸'],['puppet','🪆'],
  ]},
  { cat: '❤️', label: 'Symbols', emoji: [
    ['heart','❤️'],['orange_heart','🧡'],['yellow_heart','💛'],['green_heart','💚'],
    ['blue_heart','💙'],['purple_heart','💜'],['brown_heart','🤎'],
    ['black_heart','🖤'],['white_heart','🤍'],['broken_heart','💔'],
    ['heavy_heart_exclamation','❣️'],['two_hearts','💕'],['revolving_hearts','💞'],
    ['heartbeat','💓'],['heartpulse','💗'],['sparkling_heart','💖'],
    ['cupid','💘'],['gift_heart','💝'],['heart_decoration','💟'],
    ['peace_symbol','☮️'],['cross','✝️'],['star_and_crescent','☪️'],['star_of_david','✡️'],
    ['six_pointed_star','🔯'],['aries','♈'],['taurus','♉'],['gemini','♊'],
    ['cancer','♋'],['leo','♌'],['virgo','♍'],['libra','♎'],['scorpius','♏'],
    ['sagittarius','♐'],['capricorn','♑'],['aquarius','♒'],['pisces','♓'],
    ['id','🆔'],['atom_symbol','⚛️'],['radioactive','☢️'],['biohazard','☣️'],
    ['mobile_phone_off','📴'],['vibration_mode','📳'],['u6709','🈶'],
    ['recycle','♻️'],['fleur_de_lis','⚜️'],['beginner','🔰'],['heavy_check_mark','✔️'],
    ['ballot_box_with_check','☑️'],['radio_button','🔘'],['white_square_button','🔳'],
    ['black_square_button','🔲'],['black_small_square','▪️'],['white_small_square','▫️'],
    ['arrow_forward','▶️'],['arrow_backward','◀️'],['fast_forward','⏩'],
    ['rewind','⏪'],['twisted_rightwards_arrows','🔀'],['repeat','🔁'],
    ['repeat_one','🔂'],['arrow_right','➡️'],['arrow_left','⬅️'],['arrow_up','⬆️'],
    ['arrow_down','⬇️'],['arrow_upper_right','↗️'],['arrow_lower_right','↘️'],
    ['arrow_lower_left','↙️'],['arrow_upper_left','↖️'],['arrow_up_down','↕️'],
    ['left_right_arrow','↔️'],['arrows_counterclockwise','🔄'],
    ['arrow_right_hook','↪️'],['arrow_left_hook','↩️'],['arrow_heading_up','⤴️'],
    ['arrow_heading_down','⤵️'],['hash','#️⃣'],['asterisk','*️⃣'],
    ['zero','0️⃣'],['one','1️⃣'],['two','2️⃣'],['three','3️⃣'],['four','4️⃣'],
    ['five','5️⃣'],['six','6️⃣'],['seven','7️⃣'],['eight','8️⃣'],['nine','9️⃣'],
    ['keycap_ten','🔟'],['exclamation','❗'],['grey_exclamation','❕'],
    ['question','❓'],['grey_question','❔'],['bangbang','‼️'],['interrobang','⁉️'],
    ['100','💯'],['low_brightness','🔅'],['high_brightness','🔆'],
    ['trident','🔱'],['fleur_de_lis','⚜️'],['warning','⚠️'],
    ['zap','⚡'],['white_check_mark','✅'],['ballot_box_with_check','☑️'],
    ['x','❌'],['negative_squared_cross_mark','❎'],
    ['tada','🎉'],['sparkles','✨'],['star','⭐'],['star2','🌟'],['dizzy','💫'],
    ['fire','🔥'],['boom','💥'],['anger','💢'],['speech_balloon','💬'],
    ['thought_balloon','💭'],['zzz','💤'],['wave_dash','〰️'],
  ]},
  { cat: '🚩', label: 'Flags', emoji: [
    ['checkered_flag','🏁'],['triangular_flag_on_post','🚩'],['crossed_flags','🎌'],
    ['black_flag','🏴'],['white_flag','🏳️'],['rainbow_flag','🏳️‍🌈'],
    ['pirate_flag','🏴‍☠️'],['us','🇺🇸'],['gb','🇬🇧'],['de','🇩🇪'],['fr','🇫🇷'],
    ['es','🇪🇸'],['it','🇮🇹'],['jp','🇯🇵'],['cn','🇨🇳'],['kr','🇰🇷'],
    ['ru','🇷🇺'],['ca','🇨🇦'],['au','🇦🇺'],['br','🇧🇷'],['in','🇮🇳'],
    ['mx','🇲🇽'],['no','🇳🇴'],['se','🇸🇪'],['dk','🇩🇰'],['fi','🇫🇮'],
    ['nl','🇳🇱'],['be','🇧🇪'],['ch','🇨🇭'],['at','🇦🇹'],['pt','🇵🇹'],
    ['pl','🇵🇱'],['tr','🇹🇷'],['il','🇮🇱'],['sa','🇸🇦'],['za','🇿🇦'],
    ['ng','🇳🇬'],['eg','🇪🇬'],['ar','🇦🇷'],['cl','🇨🇱'],['co','🇨🇴'],
    ['eu','🇪🇺'],['un','🇺🇳'],
  ]},
];

const EMOJI_LOOKUP = new Map(EMOJI_DATA.flatMap(cat => cat.emoji));

function expandEmoji(text) {
  return text.replace(/:([a-zA-Z0-9_+-]+):/g, (m, n) => EMOJI_LOOKUP.get(n) ?? m);
}

function insertAtCursor(_el, text) {
  view.dispatch(view.state.replaceSelection(text));
  view.focus();
}

let emojiPickerBuilt = false;

function buildEmojiPicker() {
  if (emojiPickerBuilt) return;
  emojiPickerBuilt = true;

  const tabs = document.createElement('div');
  tabs.className = 'ep-tabs';

  const grid = document.createElement('div');
  grid.className = 'ep-grid';

  function showTab(idx) {
    tabs.querySelectorAll('.ep-tab').forEach((t, i) => t.classList.toggle('active', i === idx));
    grid.innerHTML = '';
    for (const [shortcode, char] of EMOJI_DATA[idx].emoji) {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'ep-emoji';
      btn.textContent = char;
      btn.title = `:${shortcode}:`;
      btn.addEventListener('click', () => {
        insertAtCursor(null, char);
        emojiPicker.classList.add('hidden');
      });
      grid.appendChild(btn);
    }
  }

  EMOJI_DATA.forEach((cat, idx) => {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'ep-tab';
    btn.textContent = cat.cat;
    btn.title = cat.label;
    btn.addEventListener('click', () => showTab(idx));
    tabs.appendChild(btn);
  });

  emojiPicker.appendChild(tabs);
  emojiPicker.appendChild(grid);
  showTab(0);
}

emojiBtn.addEventListener('click', e => {
  e.stopPropagation();
  buildEmojiPicker();
  const hidden = emojiPicker.classList.toggle('hidden');
  if (!hidden) {
    const r = emojiBtn.getBoundingClientRect();
    emojiPicker.style.top  = `${r.bottom + 4}px`;
    emojiPicker.style.right = `${document.documentElement.clientWidth - r.right}px`;
  }
});

document.addEventListener('click', e => {
  if (!emojiBtn.contains(e.target) && !emojiPicker.contains(e.target)) {
    emojiPicker.classList.add('hidden');
  }
});

// Toolbar action buttons
startBtn.addEventListener('click',    () => { if (selectedId) doUpdateStatus(selectedId, 'in_progress'); });
blockBtn.addEventListener('click',    () => { if (selectedId) openBlockedByModal(); });
deferBtn.addEventListener('click',    () => { if (selectedId) openDeferModal(); });
timerBtn.addEventListener('click',    () => { if (selectedId) openTimerModal(); });
closeItemBtn.addEventListener('click', () => { if (selectedId) openCloseModal(selectedId); });

deleteBtn.addEventListener('click', () => {
  if (!selectedId) return;
  openDeleteModal();
});

cleanBtn.addEventListener('click', async () => {
  clearError();
  try {
    await invoke('clean_closed', { dir: storeDir });
    if (selectedItem()?.status === 'closed') selectedId = null;
    await loadItems();
  } catch (e) {
    showError(`Clean failed: ${e}`);
  }
});

newBtn.addEventListener('click', openNewModal);
helpBtn.addEventListener('click', () => helpModal.classList.remove('hidden'));
helpModal.addEventListener('click', e => {
  if (e.target === helpModal) helpModal.classList.add('hidden');
});
nextBtn.addEventListener('click', doNext);

// Delete modal events
deleteCancelBtn.addEventListener('click', () => { deleteModal.classList.add('hidden'); });
deleteConfirmBtn.addEventListener('click', confirmDelete);
deleteModal.addEventListener('keydown', e => {
  if (e.key === 'Enter') confirmDelete();
  if (e.key === 'Escape') { deleteModal.classList.add('hidden'); e.stopPropagation(); }
});
deleteModal.addEventListener('click', e => {
  if (e.target === deleteModal) deleteModal.classList.add('hidden');
});

// Close modal events
closeCancelBtn.addEventListener('click', () => {
  closeModal.classList.add('hidden');
  pendingCloseId = '';
});
closeConfirmBtn.addEventListener('click', confirmClose);
closeReason.addEventListener('keydown', e => {
  if (e.key === 'Enter') confirmClose();
  if (e.key === 'Escape') { closeModal.classList.add('hidden'); pendingCloseId = ''; e.stopPropagation(); }
});
closeModal.addEventListener('click', e => {
  if (e.target === closeModal) { closeModal.classList.add('hidden'); pendingCloseId = ''; }
});

// New item modal events
newCancelBtn.addEventListener('click', () => { newModal.classList.add('hidden'); });
newConfirmBtn.addEventListener('click', confirmNew);
newTitle.addEventListener('keydown', e => {
  if (e.key === 'Enter') confirmNew();
  if (e.key === 'Escape') { newModal.classList.add('hidden'); e.stopPropagation(); }
});
newModal.addEventListener('click', e => {
  if (e.target === newModal) newModal.classList.add('hidden');
});

// Open Dir modal events
openDirCancelBtn.addEventListener('click', () => { openDirModal.classList.add('hidden'); });
openDirOkBtn.addEventListener('click', () => checkAndOpenDir(dirPathInput.value));
dirPathInput.addEventListener('keydown', e => {
  if (e.key === 'Enter') checkAndOpenDir(dirPathInput.value);
  if (e.key === 'Escape') { openDirModal.classList.add('hidden'); e.stopPropagation(); }
});
openDirModal.addEventListener('click', e => {
  if (e.target === openDirModal) openDirModal.classList.add('hidden');
});

dirPickBtn.addEventListener('click', async () => {
  const picked = await invoke('plugin:dialog|open', { options: { directory: true } });
  if (picked) {
    dirPathInput.value = picked;
    await checkAndOpenDir(picked);
  }
});

initYesBtn.addEventListener('click', async () => {
  const path = dirPathInput.value.trim();
  clearError();
  try {
    const crumbsDir = await invoke('init_store', { dir: path });
    openDirModal.classList.add('hidden');
    await switchStore(crumbsDir);
  } catch (e) {
    showError(`Init failed: ${e}`);
    openDirModal.classList.add('hidden');
  }
});

initNoBtn.addEventListener('click', () => {
  initPrompt.classList.add('hidden');
  openDirActions.classList.remove('hidden');
  dirPathInput.value = '';
  dirPathInput.focus();
});

// ── Init ──────────────────────────────────────────────────────────────────

rebuildTableHeader();
initColResizers();

try {
  storeDir = await invoke('resolve_store', { dir: '' });
  storePathEl.textContent = storeDir;
  addRecentStore(storeDir);
  renderSidebar();
} catch (e) {
  showError(`Could not resolve store: ${e}`);
}
await loadItems();
updateToolbarButtons();
