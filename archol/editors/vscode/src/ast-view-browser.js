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
  const expandBtn = document.getElementById('json-expand');
  const collapseBtn = document.getElementById('json-collapse');
  const statusEl = document.getElementById('status');
  const selectionEl = document.getElementById('selection');
  const bannerEl = document.getElementById('banner');
  const state = {
    ast: null,
    selectedPath: null,
    selectedNode: null,
    jsonExpanded: new Set(),
    filter: '',
    fileName: 'Archol',
    error: null,
    loading: true,
    inApplication: true,
    commentedOut: false
  };
  let statusTimer = null;

  document.addEventListener('contextmenu', function (event) {
    event.preventDefault();
  });

  function selectedNode() {
    return state.selectedNode;
  }

  function pathKey(path) {
    return path.join('/');
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

  function jsonIsExpandable(value) {
    if (value === null || typeof value !== 'object') return false;
    return Array.isArray(value) ? value.length > 0 : Object.keys(value).length > 0;
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

  function jsonMatches(value, key) {
    const needle = state.filter;
    if (!needle) return true;
    if (key !== null && String(key).toLowerCase().indexOf(needle) !== -1) return true;
    if (value === null || typeof value !== 'object') {
      return String(value).toLowerCase().indexOf(needle) !== -1;
    }
    if (Array.isArray(value)) {
      return value.some(function (item, index) { return jsonMatches(item, index); });
    }
    return Object.keys(value).some(function (childKey) {
      return jsonMatches(value[childKey], childKey);
    });
  }

  function toggleJsonPath(path) {
    const keyPath = pathKey(path);
    if (state.jsonExpanded.has(keyPath)) state.jsonExpanded.delete(keyPath);
    else state.jsonExpanded.add(keyPath);
    const scrollTop = treeEl.scrollTop;
    render();
    treeEl.scrollTop = scrollTop;
  }

  function selectNode(node) {
    if (!node) return;
    state.selectedNode = node;
    state.selectedPath = null;
    refreshSelection();
    const scrollTop = treeEl.scrollTop;
    render();
    treeEl.scrollTop = scrollTop;
    scrollSelectedRow();
  }

  function renderJsonValue(value, key, path) {
    if (!jsonMatches(value, key)) return null;
    const isNode = value !== null && typeof value === 'object' && typeof value.kind === 'string';
    const expandable = jsonIsExpandable(value);
    const expanded = expandable
      && (state.filter.length > 0 || state.jsonExpanded.has(pathKey(path)));
    const wrapper = document.createElement('div');
    const row = document.createElement('div');
    row.className = 'json-row' + (expandable ? ' expandable' : '') + (isNode ? ' ast-node' : '');
    if (isNode && value === state.selectedNode) row.classList.add('selected');
    row.dataset.path = pathKey(path);

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
      preview.textContent = orderedJson(value);
      row.appendChild(preview);
    } else {
      const open = document.createElement('span');
      open.className = 'json-closing';
      open.textContent = Array.isArray(value) ? '[' : '{';
      row.appendChild(open);
    }

    if (expandable) {
      twisty.addEventListener('click', function (event) {
        event.stopPropagation();
        toggleJsonPath(path);
      });
      row.addEventListener('click', function (event) {
        event.stopPropagation();
        if (isNode) {
          selectNode(value);
          if (value.span && Number.isFinite(value.span.start)) revealSpan(value.span, false);
        } else {
          toggleJsonPath(path);
        }
      });
    } else if (isNode) {
      row.addEventListener('click', function (event) {
        event.stopPropagation();
        selectNode(value);
        if (value.span && Number.isFinite(value.span.start)) revealSpan(value.span, false);
      });
    }
    wrapper.appendChild(row);

    if (expandable && expanded) {
      const childrenWrap = document.createElement('div');
      childrenWrap.className = 'json-children';
      if (Array.isArray(value)) {
        value.forEach(function (item, index) {
          const child = renderJsonValue(item, index, path.concat(index));
          if (child) childrenWrap.appendChild(child);
        });
      } else {
        orderedKeys(value).forEach(function (childKey) {
          const child = renderJsonValue(value[childKey], childKey, path.concat(childKey));
          if (child) childrenWrap.appendChild(child);
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

  function emptyMessage(text, isError) {
    const paragraph = document.createElement('p');
    paragraph.className = 'empty' + (isError ? ' error' : '');
    paragraph.textContent = text;
    treeEl.appendChild(paragraph);
  }

  function renderJsonRoot() {
    const value = state.ast;
    if (!value) return;
    if (!jsonMatches(value, null)) {
      emptyMessage('No JSON matches the filter "' + state.filter + '".');
      return;
    }
    const row = renderJsonValue(value, null, []);
    if (row) treeEl.appendChild(row);
  }

  function scrollSelectedRow() {
    const selected = treeEl.querySelector('.json-row.selected');
    if (selected) selected.scrollIntoView({ block: 'center', inline: 'nearest' });
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
    renderJsonRoot();
    updateStatus();
    refreshSelection();
  }

  function findJsonPath(value, target, path) {
    if (value === target) return path;
    if (value === null || typeof value !== 'object') return null;
    if (Array.isArray(value)) {
      for (let index = 0; index < value.length; index += 1) {
        const found = findJsonPath(value[index], target, path.concat(index));
        if (found) return found;
      }
      return null;
    }
    const keys = orderedKeys(value);
    for (let index = 0; index < keys.length; index += 1) {
      const found = findJsonPath(value[keys[index]], target, path.concat(keys[index]));
      if (found) return found;
    }
    return null;
  }

  function expandAll() {
    state.jsonExpanded = new Set();
    (function collect(value, path) {
      if (!jsonIsExpandable(value)) return;
      state.jsonExpanded.add(pathKey(path));
      if (Array.isArray(value)) {
        value.forEach(function (item, index) { collect(item, path.concat(index)); });
      } else {
        orderedKeys(value).forEach(function (key) { collect(value[key], path.concat(key)); });
      }
    })(state.ast, []);
    const scrollTop = treeEl.scrollTop;
    render();
    treeEl.scrollTop = scrollTop;
  }

  function collapseAll() {
    state.jsonExpanded = new Set();
    const scrollTop = treeEl.scrollTop;
    render();
    treeEl.scrollTop = Math.min(scrollTop, treeEl.scrollHeight);
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
      focus: Boolean(focus)
    });
  }

  filterEl.addEventListener('input', function () {
    state.filter = filterEl.value.trim().toLowerCase();
    render();
  });

  revealBtn.addEventListener('click', function () {
    const node = selectedNode();
    if (!node) {
      setTransientStatus('Select an AST node');
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
    const text = orderedJsonPretty(node, 2);
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

  expandBtn.addEventListener('click', expandAll);
  collapseBtn.addEventListener('click', collapseAll);

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
      state.selectedNode = state.selectedPath && state.ast
        ? nodeAtPath(state.ast, state.selectedPath)
        : null;
      state.jsonExpanded = new Set();
      if (message.revealSelected && state.selectedNode) {
        const selectedJsonPath = findJsonPath(state.ast, state.selectedNode, []);
        if (selectedJsonPath) {
          for (let size = 0; size <= selectedJsonPath.length; size += 1) {
            state.jsonExpanded.add(pathKey(selectedJsonPath.slice(0, size)));
          }
        }
      }
      fileEl.textContent = state.fileName;
      fileEl.title = message.uri || state.fileName;
      updateBanner();
      render();
      if (message.revealSelected) scrollSelectedRow();
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
