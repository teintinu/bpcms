'use strict';

const { browserUtils } = require('./ast-view-browser-utils');
const { browserNodeHelpersSource } = require('./app-support');

const browserScript = `(function () {
  'use strict';
${browserUtils}
${browserNodeHelpersSource()}
  const vscode = acquireVsCodeApi();
  const treeEl = document.getElementById('tree');
  const fileEl = document.getElementById('file-name');
  const filterEl = document.getElementById('filter');
  const revealBtn = document.getElementById('reveal');
  const declarationBtn = document.getElementById('declaration');
  const usagesBtn = document.getElementById('usages');
  const copyBtn = document.getElementById('copy');
  const jsonPopoverEl = document.getElementById('json-popover');
  const jsonTitleEl = document.getElementById('json-title');
  const jsonTreeEl = document.getElementById('json-tree');
  const jsonExpandBtn = document.getElementById('json-expand');
  const jsonCollapseBtn = document.getElementById('json-collapse');
  const statusEl = document.getElementById('status');
  const selectionEl = document.getElementById('selection');
  const bannerEl = document.getElementById('banner');
  const state = {
    ast: null,
    selectedPath: null,
    expanded: new Set(['']),
    jsonExpanded: new Set(),
    filter: '',
    fileName: 'Archol',
    error: null,
    loading: true,
    inApplication: true,
    commentedOut: false
  };
  let statusTimer = null;
  let jsonHoverTimer = null;
  let jsonHideTimer = null;
  let jsonNode = null;
  let jsonAnchor = null;
  const JSON_HOVER_DELAY_MS = 320;
  const JSON_HIDE_DELAY_MS = 120;
  const JSON_GAP_PX = 8;
  const JSON_MARGIN_PX = 8;

  document.addEventListener('contextmenu', function (event) {
    event.preventDefault();
  });

  function selectedNode() {
    return state.selectedPath ? nodeAtPath(state.selectedPath) : null;
  }

  function jsonIsExpandable(value) {
    if (value === null || typeof value !== 'object') return false;
    return Array.isArray(value) ? value.length > 0 : Object.keys(value).length > 0;
  }

  function jsonInline(value) {
    try {
      return JSON.stringify(value);
    } catch (error) {
      return String(value);
    }
  }

  function appendJsonScalar(parent, value) {
    const span = document.createElement('span');
    if (value === null) {
      span.className = 'json-null';
      span.textContent = 'null';
    } else if (typeof value === 'string') {
      span.className = 'json-string';
      span.textContent = JSON.stringify(value);
    } else if (typeof value === 'number') {
      span.className = 'json-number';
      span.textContent = String(value);
    } else if (typeof value === 'boolean') {
      span.className = 'json-boolean';
      span.textContent = String(value);
    } else {
      span.textContent = String(value);
    }
    parent.appendChild(span);
  }

  function renderJsonValue(value, key, path) {
    const wrapper = document.createElement('div');
    const row = document.createElement('div');
    const expandable = jsonIsExpandable(value);
    const expanded = expandable && state.jsonExpanded.has(pathKey(path));
    row.className = 'json-row' + (expandable ? ' expandable' : '');

    const twisty = document.createElement('span');
    twisty.className = 'json-twisty';
    twisty.textContent = expandable ? (expanded ? '▾' : '▸') : ' ';
    row.appendChild(twisty);

    if (key !== null) {
      const keyEl = document.createElement('span');
      keyEl.className = 'json-key';
      keyEl.textContent = JSON.stringify(String(key)) + ': ';
      row.appendChild(keyEl);
    }

    if (!expandable) {
      if (value !== null && typeof value === 'object') {
        const preview = document.createElement('span');
        preview.className = 'json-preview';
        preview.textContent = Array.isArray(value) ? '[ ]' : '{ }';
        row.appendChild(preview);
      } else {
        appendJsonScalar(row, value);
      }
    } else if (!expanded) {
      const preview = document.createElement('span');
      preview.className = 'json-preview';
      preview.textContent = jsonInline(value);
      row.appendChild(preview);
    } else {
      const open = document.createElement('span');
      open.className = 'json-closing';
      open.textContent = Array.isArray(value) ? '[' : '{';
      row.appendChild(open);
    }

    if (expandable) {
      row.addEventListener('click', function (event) {
        event.stopPropagation();
        const keyPath = pathKey(path);
        if (state.jsonExpanded.has(keyPath)) state.jsonExpanded.delete(keyPath);
        else state.jsonExpanded.add(keyPath);
        rerenderJsonPopover();
      });
    }
    wrapper.appendChild(row);

    if (expandable && expanded) {
      const childrenWrap = document.createElement('div');
      childrenWrap.className = 'json-children';
      if (Array.isArray(value)) {
        value.forEach(function (item, index) {
          childrenWrap.appendChild(renderJsonValue(item, index, path.concat(index)));
        });
      } else {
        Object.keys(value).forEach(function (childKey) {
          childrenWrap.appendChild(renderJsonValue(value[childKey], childKey, path.concat(childKey)));
        });
      }
      wrapper.appendChild(childrenWrap);
      const closeRow = document.createElement('div');
      closeRow.className = 'json-row';
      closeRow.style.paddingLeft = '16px';
      const close = document.createElement('span');
      close.className = 'json-closing';
      close.textContent = Array.isArray(value) ? ']' : '}';
      closeRow.appendChild(close);
      wrapper.appendChild(closeRow);
    }
    return wrapper;
  }

  function renderJsonRoot(value) {
    jsonTreeEl.textContent = '';
    if (!jsonIsExpandable(value)) {
      const row = document.createElement('div');
      row.className = 'json-row';
      const spacer = document.createElement('span');
      spacer.className = 'json-twisty';
      spacer.textContent = ' ';
      row.appendChild(spacer);
      if (value !== null && typeof value === 'object') {
        const preview = document.createElement('span');
        preview.className = 'json-preview';
        preview.textContent = Array.isArray(value) ? '[ ]' : '{ }';
        row.appendChild(preview);
      } else {
        appendJsonScalar(row, value);
      }
      jsonTreeEl.appendChild(row);
      return;
    }
    if (Array.isArray(value)) {
      value.forEach(function (item, index) {
        jsonTreeEl.appendChild(renderJsonValue(item, index, [index]));
      });
    } else {
      Object.keys(value).forEach(function (key) {
        jsonTreeEl.appendChild(renderJsonValue(value[key], key, [key]));
      });
    }
  }

  function renderJsonPopover(node) {
    if (!node) return;
    const name = node.name ? ' ' + node.name : '';
    jsonTitleEl.textContent = node.kind + name;
    renderJsonRoot(node);
  }

  function placeJsonPopover(anchor) {
    jsonPopoverEl.hidden = false;
    const rect = anchor.getBoundingClientRect();
    const popover = jsonPopoverEl.getBoundingClientRect();
    const maxLeft = window.innerWidth - popover.width - JSON_MARGIN_PX;
    const maxTop = window.innerHeight - popover.height - JSON_MARGIN_PX;
    let left = rect.right + JSON_GAP_PX;
    if (left > maxLeft) left = rect.left - JSON_GAP_PX - popover.width;
    if (left < JSON_MARGIN_PX) left = Math.max(JSON_MARGIN_PX, maxLeft);
    let top = rect.top;
    if (top > maxTop) top = maxTop;
    if (top < JSON_MARGIN_PX) top = JSON_MARGIN_PX;
    jsonPopoverEl.style.left = Math.round(left) + 'px';
    jsonPopoverEl.style.top = Math.round(top) + 'px';
  }

  function showJsonPopover(node, anchor) {
    if (!node || !anchor) return;
    if (jsonHideTimer) {
      clearTimeout(jsonHideTimer);
      jsonHideTimer = null;
    }
    jsonNode = node;
    jsonAnchor = anchor;
    state.jsonExpanded = new Set();
    renderJsonPopover(node);
    placeJsonPopover(anchor);
  }

  function hideJsonPopover() {
    if (jsonHoverTimer) {
      clearTimeout(jsonHoverTimer);
      jsonHoverTimer = null;
    }
    if (jsonHideTimer) {
      clearTimeout(jsonHideTimer);
      jsonHideTimer = null;
    }
    jsonPopoverEl.hidden = true;
    jsonNode = null;
    jsonAnchor = null;
  }

  function scheduleJsonPopover(node, anchor) {
    if (jsonHoverTimer) clearTimeout(jsonHoverTimer);
    if (jsonHideTimer) {
      clearTimeout(jsonHideTimer);
      jsonHideTimer = null;
    }
    jsonHoverTimer = setTimeout(function () {
      jsonHoverTimer = null;
      showJsonPopover(node, anchor);
    }, JSON_HOVER_DELAY_MS);
  }

  function scheduleHideJsonPopover() {
    if (jsonHoverTimer) {
      clearTimeout(jsonHoverTimer);
      jsonHoverTimer = null;
    }
    if (jsonHideTimer) clearTimeout(jsonHideTimer);
    jsonHideTimer = setTimeout(function () {
      jsonHideTimer = null;
      hideJsonPopover();
    }, JSON_HIDE_DELAY_MS);
  }

  function rerenderJsonPopover() {
    if (!jsonNode) return;
    renderJsonPopover(jsonNode);
    if (jsonAnchor) placeJsonPopover(jsonAnchor);
  }

  function navigationEnabled() {
    return state.inApplication && !state.commentedOut;
  }

  function refreshSelection() {
    const node = selectedNode();
    const summary = node ? referenceSummary(node) : '';
    selectionEl.textContent = summary;
    selectionEl.title = summary ? 'Selected node identifiers: ' + summary : '';
    const enabled = navigationEnabled();
    const declarationId = enabled ? declarationTargetId(node) : null;
    const usageId = enabled ? usageTargetId(node) : null;
    declarationBtn.disabled = declarationId === null || declarationId === undefined;
    usagesBtn.disabled = usageId === null || usageId === undefined;
  }

  function updateBanner() {
    bannerEl.hidden = navigationEnabled();
  }

  function matches(node) {
    if (!state.filter) return true;
    const haystack = (node.kind + ' ' + summaryOf(node)).toLowerCase();
    if (haystack.indexOf(state.filter) !== -1) return true;
    return childrenOf(node).some(matches);
  }

  function updateStatus() {
    if (statusTimer) return;
    if (state.error) {
      statusEl.textContent = 'error';
      return;
    }
    statusEl.textContent = state.ast ? countNodes(state.ast) + ' nodes' : '';
  }

  function setTransientStatus(text) {
    statusEl.textContent = text;
    if (statusTimer) clearTimeout(statusTimer);
    statusTimer = setTimeout(function () {
      statusTimer = null;
      updateStatus();
    }, 2200);
  }

  function expandAncestors(path) {
    for (let size = 1; size < path.length; size += 1) {
      state.expanded.add(pathKey(path.slice(0, size)));
    }
  }

  function selectPath(path) {
    state.selectedPath = path;
    const key = pathKey(path);
    const rows = treeEl.querySelectorAll('.node-row');
    for (let index = 0; index < rows.length; index += 1) {
      rows[index].classList.toggle('selected', rows[index].dataset.path === key);
      if (rows[index].dataset.path === key) rows[index].scrollIntoView({ block: 'nearest' });
    }
    refreshSelection();
  }

  function revealSpan(span, focus) {
    if (!span || !Number.isFinite(span.start)) {
      setTransientStatus('Node has no code position');
      return;
    }
    vscode.postMessage({
      type: 'reveal',
      start: span.start,
      end: Number.isFinite(span.end) ? span.end : span.start,
      path: state.selectedPath,
      focus: Boolean(focus)
    });
  }

  function makeRow(node, path) {
    const children = childrenOf(node);
    const filterMode = state.filter.length > 0;
    const expanded = filterMode || path.length === 0 || state.expanded.has(pathKey(path));
    const row = document.createElement('div');
    row.className = 'node-row';
    row.dataset.path = pathKey(path);
    row.setAttribute('role', 'treeitem');
    row.setAttribute('aria-level', String(path.length + 1));
    row.setAttribute('aria-expanded', children.length > 0 ? String(expanded) : 'false');
    if (state.selectedPath && pathKey(state.selectedPath) === pathKey(path)) {
      row.classList.add('selected');
    }

    const twisty = document.createElement('span');
    twisty.className = 'twisty' + (children.length > 0 ? '' : ' leaf');
    twisty.textContent = expanded ? '▾' : '▸';
    twisty.setAttribute('aria-hidden', 'true');
    row.appendChild(twisty);

    const kind = document.createElement('span');
    kind.className = 'node-kind cat-' + categoryOf(node.kind);
    appendHighlighted(kind, node.kind, state.filter);
    row.appendChild(kind);

    const summary = summaryOf(node);
    if (summary) {
      const summaryEl = document.createElement('span');
      summaryEl.className = 'node-summary';
      appendHighlighted(summaryEl, summary, state.filter);
      row.appendChild(summaryEl);
    }

    row.addEventListener('mouseenter', function () {
      scheduleJsonPopover(node, row);
    });
    row.addEventListener('mouseleave', function () {
      scheduleHideJsonPopover();
    });

    row.addEventListener('click', function (event) {
      if (event.target === twisty && children.length > 0) {
        if (!filterMode) {
          const key = pathKey(path);
          if (state.expanded.has(key)) state.expanded.delete(key);
          else state.expanded.add(key);
          render();
        }
        return;
      }
      state.selectedPath = path;
      selectPath(path);
      if (node.span && Number.isFinite(node.span.start)) revealSpan(node.span, false);
    });
    return row;
  }

  function renderNode(node, path) {
    const wrapper = document.createElement('div');
    wrapper.appendChild(makeRow(node, path));
    const children = childrenOf(node);
    const filterMode = state.filter.length > 0;
    const expanded = filterMode || path.length === 0 || state.expanded.has(pathKey(path));
    if (children.length > 0 && expanded) {
      const childWrap = document.createElement('div');
      childWrap.className = 'children';
      childWrap.setAttribute('role', 'group');
      children.forEach(function (child, index) {
        if (filterMode && !matches(child)) return;
        childWrap.appendChild(renderNode(child, path.concat(index)));
      });
      wrapper.appendChild(childWrap);
    }
    return wrapper;
  }

  function emptyMessage(text, isError) {
    const paragraph = document.createElement('p');
    paragraph.className = 'empty' + (isError ? ' error' : '');
    paragraph.textContent = text;
    treeEl.appendChild(paragraph);
  }

  function render() {
    hideJsonPopover();
    treeEl.textContent = '';
    if (state.error) {
      emptyMessage(state.error, true);
      updateStatus();
      refreshSelection();
      return;
    }
    if (!state.ast) {
      emptyMessage(state.loading ? 'Loading AST…' : 'No AST received.');
      updateStatus();
      refreshSelection();
      return;
    }
    if (state.filter && !matches(state.ast)) {
      emptyMessage('No node matches the filter "' + state.filter + '".');
      updateStatus();
      refreshSelection();
      return;
    }
    treeEl.appendChild(renderNode(state.ast, []));
    updateStatus();
    refreshSelection();
    const selectedRow = treeEl.querySelector('.node-row.selected');
    if (selectedRow) selectedRow.scrollIntoView({ block: 'nearest' });
  }

  filterEl.addEventListener('input', function () {
    state.filter = filterEl.value.trim().toLowerCase();
    render();
  });

  revealBtn.addEventListener('click', function () {
    const node = selectedNode();
    if (!node) {
      setTransientStatus('Select a node');
      return;
    }
    if (!node.span || !Number.isFinite(node.span.start)) {
      setTransientStatus('Node has no code position');
      return;
    }
    revealSpan(node.span, true);
  });

  declarationBtn.addEventListener('click', function () {
    const node = selectedNode();
    if (!navigationEnabled()) return;
    const id = declarationTargetId(node);
    if (id === null || id === undefined) {
      setTransientStatus('Select a reference or declaration');
      return;
    }
    setTransientStatus('Opening declaration…');
    vscode.postMessage({ type: 'declaration', id: id });
  });

  usagesBtn.addEventListener('click', function () {
    const node = selectedNode();
    if (!navigationEnabled()) return;
    const id = usageTargetId(node);
    if (id === null || id === undefined) {
      setTransientStatus('Select a declaration');
      return;
    }
    setTransientStatus('Finding usages…');
    vscode.postMessage({ type: 'usages', id: id });
  });

  function fallbackCopy(text) {
    const area = document.createElement('textarea');
    area.value = text;
    area.setAttribute('readonly', 'readonly');
    area.style.position = 'fixed';
    area.style.opacity = '0';
    document.body.appendChild(area);
    area.select();
    const copied = document.execCommand('copy');
    document.body.removeChild(area);
    return copied;
  }

  copyBtn.addEventListener('click', function () {
    const node = selectedNode() || state.ast;
    if (!node) return;
    const text = JSON.stringify(node, null, 2);
    const done = function (ok) {
      setTransientStatus(ok ? 'JSON copied' : 'Failed to copy');
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(function () { done(true); }, function () {
        done(fallbackCopy(text));
      });
    } else {
      done(fallbackCopy(text));
    }
  });

  jsonPopoverEl.addEventListener('mouseenter', function () {
    if (jsonHideTimer) {
      clearTimeout(jsonHideTimer);
      jsonHideTimer = null;
    }
  });
  jsonPopoverEl.addEventListener('mouseleave', function () {
    scheduleHideJsonPopover();
  });

  jsonExpandBtn.addEventListener('click', function () {
    if (!jsonNode) return;
    state.jsonExpanded = new Set();
    (function collect(value, path) {
      if (!jsonIsExpandable(value)) return;
      state.jsonExpanded.add(pathKey(path));
      if (Array.isArray(value)) {
        value.forEach(function (item, index) { collect(item, path.concat(index)); });
      } else {
        Object.keys(value).forEach(function (key) { collect(value[key], path.concat(key)); });
      }
    })(jsonNode, []);
    rerenderJsonPopover();
  });

  jsonCollapseBtn.addEventListener('click', function () {
    state.jsonExpanded = new Set();
    rerenderJsonPopover();
  });

  treeEl.addEventListener('scroll', hideJsonPopover);
  window.addEventListener('resize', hideJsonPopover);

  window.addEventListener('message', function (event) {
    const message = event.data;
    if (!message || typeof message.type !== 'string') return;
    if (message.type === 'setAst') {
      state.ast = message.ast || null;
      state.error = null;
      state.loading = false;
      state.fileName = message.fileName || 'Archol';
      state.inApplication = message.inApplication !== false;
      state.commentedOut = Boolean(message.commentedOut);
      state.selectedPath = Array.isArray(message.selectedPath) ? message.selectedPath : null;
      state.expanded = new Set(['']);
      if (state.ast) {
        childrenOf(state.ast).forEach(function (_, index) { state.expanded.add(String(index)); });
      }
      state.jsonExpanded = new Set();
      hideJsonPopover();
      if (state.selectedPath) expandAncestors(state.selectedPath);
      fileEl.textContent = state.fileName;
      fileEl.title = message.uri || state.fileName;
      updateBanner();
      render();
      return;
    }
    if (message.type === 'loading') {
      state.loading = true;
      state.error = null;
      state.inApplication = true;
      state.commentedOut = false;
      updateBanner();
      render();
      return;
    }
    if (message.type === 'error') {
      state.error = String(message.message || 'Failed to load the AST.');
      state.loading = false;
      state.inApplication = true;
      state.commentedOut = false;
      updateBanner();
      render();
    }
  });

  updateBanner();
  updateStatus();
  refreshSelection();
  vscode.postMessage({ type: 'ready' });
})();`;

module.exports = { browserScript };
