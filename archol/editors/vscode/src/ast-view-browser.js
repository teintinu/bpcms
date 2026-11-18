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
  const statusEl = document.getElementById('status');
  const selectionEl = document.getElementById('selection');
  const bannerEl = document.getElementById('banner');
  const state = {
    ast: null,
    selectedPath: null,
    expanded: new Set(['']),
    filter: '',
    fileName: 'Archol',
    error: null,
    loading: true,
    inApplication: true,
    commentedOut: false
  };
  let statusTimer = null;

  function selectedNode() {
    return state.selectedPath ? nodeAtPath(state.selectedPath) : null;
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

    if (node.span && Number.isFinite(node.span.start)) {
      row.title = node.kind + ' • offsets ' + node.span.start + '–' + node.span.end;
    }

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
