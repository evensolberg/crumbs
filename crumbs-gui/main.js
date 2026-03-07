const { invoke } = globalThis.__TAURI__.core;

// ── State ─────────────────────────────────────────────────────────────────

let storeDir = '';
let allItems = [];
let selectedId = null;
let filterStatus   = 'all';
let filterPriority = 'any';
let filterType     = 'any';
let filterTag      = '';
let previewMode = false;
let pendingCloseId = '';
let sortCol = 'priority';
let sortDir = 'asc';

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
const listPane         = document.getElementById('list-pane');
const detailPane       = document.getElementById('detail-pane');
const resizeHandle     = document.getElementById('resize-handle');
const propGrid         = document.getElementById('prop-grid');
const detailActions    = document.getElementById('detail-actions');
const detailTitleLabel = document.getElementById('detail-title-label');
const detailText       = document.getElementById('detail-text');
const detailPreview    = document.getElementById('detail-preview');
const previewBtn       = document.getElementById('preview-btn');

// Toolbar action buttons
const openDirBtn    = document.getElementById('open-dir-btn');
const newBtn        = document.getElementById('new-btn');
const startBtn      = document.getElementById('start-btn');
const blockBtn      = document.getElementById('block-btn');
const deferBtn      = document.getElementById('defer-btn');
const closeItemBtn  = document.getElementById('close-item-btn');
const deleteBtn     = document.getElementById('delete-btn');
const cleanBtn      = document.getElementById('clean-btn');

// Close modal
const closeModal      = document.getElementById('close-modal');
const closeReason     = document.getElementById('close-reason');
const closeCancelBtn  = document.getElementById('close-cancel-btn');
const closeConfirmBtn = document.getElementById('close-confirm-btn');

// New item modal
const newModal      = document.getElementById('new-modal');
const newTitle      = document.getElementById('new-title');
const newCancelBtn  = document.getElementById('new-cancel-btn');
const newConfirmBtn = document.getElementById('new-confirm-btn');

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
  blocked: '⊘ Blocked',
  deferred: '◷ Deferred',
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

// ── Toolbar contextual button state ──────────────────────────────────────

function updateToolbarButtons() {
  const item = selectedItem();
  const hasSelection = item !== null;
  const isClosed = item?.status === 'closed';

  startBtn.disabled    = !hasSelection || isClosed || item?.status === 'in_progress';
  blockBtn.disabled    = !hasSelection || isClosed || item?.status === 'blocked';
  deferBtn.disabled    = !hasSelection || isClosed || item?.status === 'deferred';
  closeItemBtn.disabled = !hasSelection || isClosed;
  deleteBtn.disabled   = !hasSelection;
}

// ── Vertical resize ───────────────────────────────────────────────────────

(function initResize() {
  let dragging = false;
  let startY = 0;
  let startListH = 0;

  resizeHandle.addEventListener('mousedown', e => {
    dragging = true;
    startY = e.clientY;
    startListH = listPane.getBoundingClientRect().height;
    resizeHandle.classList.add('dragging');
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'row-resize';
  });

  document.addEventListener('mousemove', e => {
    if (!dragging) return;
    const delta = e.clientY - startY;
    const newH = Math.max(80, startListH + delta);
    listPane.style.flex = 'none';
    listPane.style.height = `${newH}px`;
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
  const items = filteredItems();
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
      default:             av = a.priority; bv = b.priority;
    }
    if (av < bv) return -dir;
    if (av > bv) return  dir;
    return 0;
  });
}

function cellFor(item, colKey) {
  switch (colKey) {
    case 'id':           return `<td class="item-id">${escHtml(item.id)}</td>`;
    case 'title':        return `<td class="item-title">${escHtml(item.title)}</td>`;
    case 'status':       return `<td>${statusBadge(item.status)}</td>`;
    case 'type':         return `<td style="font-size:11px;color:var(--text-dim)">${escHtml(item.type ?? '')}</td>`;
    case 'priority':     return `<td>${priorityBadge(item.priority)}</td>`;
    case 'due':          return `<td>${dueHtml(item.due)}</td>`;
    case 'tags':         return `<td class="item-tags">${escHtml((item.tags ?? []).join(', '))}</td>`;
    case 'story_points': return `<td style="text-align:center;font-size:11px;color:var(--text-dim)">${item.story_points != null ? item.story_points : '—'}</td>`;
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

function renderTable() {
  const items = sortedItems();
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
      e.stopPropagation();
      startX = e.clientX;
      startW = th.getBoundingClientRect().width;
      handle.classList.add('resizing');
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';

      function onMove(e) {
        const w = Math.max(40, startW + e.clientX - startX);
        cols[i].style.width = `${w}px`;
      }
      function onUp() {
        handle.classList.remove('resizing');
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
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

  const dueInput = document.createElement('input');
  dueInput.type = 'date';
  dueInput.value = item.due ?? '';
  dueInput.addEventListener('change', () => doUpdateDue(item.id, dueInput.value));
  propRow('Due', '').appendChild(dueInput);

  if ((item.tags ?? []).length > 0) {
    propRow('Tags', escHtml(item.tags.join(', ')));
  }
  if ((item.dependencies ?? []).length > 0) {
    propRow('Depends', escHtml(item.dependencies.join(', ')));
  }
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
  detailText.classList.toggle('hidden', on);
  detailPreview.classList.toggle('hidden', !on);
  if (on) {
    detailPreview.innerHTML = marked.parse(detailText.value || '');
  }
}

function renderDetail(item) {
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
  detailText.value = item.description ?? '';
  setPreviewMode(false);
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
    await loadItems();
  } catch (e) {
    showError(`Save failed: ${e}`);
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
    if (e.key === 'Escape') { input.removeEventListener('blur', commit); renderSidebar(); }
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
  if (e.key === 'Escape') hideContextMenu();
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
  selectedId = tr.dataset.id;
  for (const r of document.querySelectorAll('#items-body tr')) {
    r.classList.remove('selected');
  }
  tr.classList.add('selected');
  renderDetail(selectedItem());
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
    if (e.key === 'Escape') { input.removeEventListener('blur', commit); input.replaceWith(detailTitleLabel); }
  });
});

// Body text autosave on blur
detailText.addEventListener('blur', () => {
  if (selectedId) doSaveText(selectedId, detailText.value);
});

previewBtn.addEventListener('click', () => setPreviewMode(!previewMode));

// Toolbar action buttons
startBtn.addEventListener('click',    () => { if (selectedId) doUpdateStatus(selectedId, 'in_progress'); });
blockBtn.addEventListener('click',    () => { if (selectedId) doUpdateStatus(selectedId, 'blocked'); });
deferBtn.addEventListener('click',    () => { if (selectedId) doUpdateStatus(selectedId, 'deferred'); });
closeItemBtn.addEventListener('click', () => { if (selectedId) openCloseModal(selectedId); });

deleteBtn.addEventListener('click', async () => {
  if (!selectedId) return;
  clearError();
  try {
    await invoke('delete_item', { dir: storeDir, id: selectedId });
    selectedId = null;
    await loadItems();
  } catch (e) {
    showError(`Delete failed: ${e}`);
  }
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
openDirBtn.addEventListener('click', openOpenDirModal);

// Close modal events
closeCancelBtn.addEventListener('click', () => {
  closeModal.classList.add('hidden');
  pendingCloseId = '';
});
closeConfirmBtn.addEventListener('click', confirmClose);
closeReason.addEventListener('keydown', e => {
  if (e.key === 'Enter') confirmClose();
  if (e.key === 'Escape') { closeModal.classList.add('hidden'); pendingCloseId = ''; }
});
closeModal.addEventListener('click', e => {
  if (e.target === closeModal) { closeModal.classList.add('hidden'); pendingCloseId = ''; }
});

// New item modal events
newCancelBtn.addEventListener('click', () => { newModal.classList.add('hidden'); });
newConfirmBtn.addEventListener('click', confirmNew);
newTitle.addEventListener('keydown', e => {
  if (e.key === 'Enter') confirmNew();
  if (e.key === 'Escape') newModal.classList.add('hidden');
});
newModal.addEventListener('click', e => {
  if (e.target === newModal) newModal.classList.add('hidden');
});

// Open Dir modal events
openDirCancelBtn.addEventListener('click', () => { openDirModal.classList.add('hidden'); });
openDirOkBtn.addEventListener('click', () => checkAndOpenDir(dirPathInput.value));
dirPathInput.addEventListener('keydown', e => {
  if (e.key === 'Enter') checkAndOpenDir(dirPathInput.value);
  if (e.key === 'Escape') openDirModal.classList.add('hidden');
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
