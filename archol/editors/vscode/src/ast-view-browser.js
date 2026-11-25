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
  const backBtn = document.getElementById('back');
  const forwardBtn = document.getElementById('forward');
  const favoriteBtn = document.getElementById('favorite');
  const favoritesEl = document.getElementById('favorites');
  const refPopup = document.getElementById('ref-popup');
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
    commentedOut: false,
    applicationMode: false,
    artifactIndex: new Map(),
    history: [],
    historyIndex: -1,
    favorites: []
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

  function valueAtJsonPath(root, path) {
    let node = root;
    if (!Array.isArray(path)) return null;
    for (let index = 0; index < path.length; index += 1) {
      if (node === null || typeof node !== 'object') return null;
      node = node[path[index]];
      if (node === undefined) return null;
    }
    return node;
  }

  function isReferenceKind(kind) {
    return typeof kind === 'string'
      && kind.endsWith('Ref')
      && kind !== 'StdRef'
      && kind !== 'PrimitiveRef'
      && kind !== 'ThemeRef'
      && kind !== 'LocaleRef'
      && kind !== 'VariableRef';
  }

  function buildAstIndex(ast) {
    state.artifactIndex = new Map();
    if (!ast || typeof ast !== 'object') return;
    const artifacts = ast.artifact && typeof ast.artifact === 'object' ? ast.artifact : null;
    if (artifacts) {
      Object.keys(artifacts).forEach(function (key) {
        const definition = artifacts[key];
        if (definition && typeof definition === 'object') {
          state.artifactIndex.set(key, definition);
          if (typeof definition.id === 'string') state.artifactIndex.set(definition.id, definition);
        }
      });
    }
  }

  function resolveReference(node) {
    if (!node || typeof node !== 'object' || !isReferenceKind(node.kind)) return null;
    if (node.kind === 'FieldRef') {
      const document = state.artifactIndex.get(node.documentId);
      const fields = document && Array.isArray(document.fields) ? document.fields : [];
      return fields.find(function (field) { return field && field.name === node.fieldPath; }) || null;
    }
    if (node.kind === 'ShapeValueRef') {
      const shape = state.artifactIndex.get(node.shapeId);
      const values = shape && Array.isArray(shape.values) ? shape.values : [];
      return values.find(function (value) { return value && value.name === node.valuePath; }) || null;
    }
    if (node.kind === 'StepRef') {
      const process = state.artifactIndex.get(node.processId);
      const steps = process && Array.isArray(process.steps) ? process.steps : [];
      return steps.find(function (step) { return step && step.name === node.stepPath; }) || null;
    }
    if (node.kind === 'ActionRef') {
      const document = state.artifactIndex.get(node.documentId);
      const actions = document && Array.isArray(document.actions) ? document.actions : [];
      return actions.find(function (action) { return action && action.name === node.actionPath; }) || null;
    }
    if (typeof node.id === 'string') {
      return state.artifactIndex.get(node.id) || null;
    }
    return null;
  }

  function scalarText(value) {
    if (typeof value === 'string') return value;
    if (typeof value === 'number' || typeof value === 'boolean') return String(value);
    if (value && typeof value === 'object' && value.kind === 'ShapeValue' && value.value) {
      return Object.keys(value.value).map(function (locale) {
        return locale + ': ' + value.value[locale];
      }).join(' / ');
    }
    return '';
  }

  function referencePopupText(node) {
    const lines = [];
    const name = node.name || node.path || '';
    lines.push(String(node.kind) + (name ? '  ' + name : ''));
    if (node.id) lines.push('id: ' + node.id);
    ['title', 'value', 'message', 'refKind', 'fieldPath', 'valuePath', 'stepPath', 'actionPath'].forEach(function (key) {
      const text = scalarText(node[key]);
      if (text) lines.push(key + ': ' + text);
    });
    if (node.loc && node.loc.file) {
      const line = Number.isFinite(node.loc.line) ? ':' + node.loc.line : '';
      let file = node.loc.file;
      if (state.ast && Array.isArray(state.ast.files)) {
        const entry = state.ast.files.find(function (candidate) {
          return candidate && candidate.id === file;
        });
        if (entry && typeof entry.path === 'string') file = entry.path;
      }
      lines.push(file + line);
    }
    return lines.join('\\n');
  }

  function showReferencePopup(node, event) {
    if (!refPopup || !node) return;
    refPopup.textContent = referencePopupText(node);
    refPopup.hidden = false;
    const padding = 12;
    let x = (event && event.clientX ? event.clientX : 0) + padding;
    let y = (event && event.clientY ? event.clientY : 0) + padding;
    const width = refPopup.offsetWidth || 0;
    const height = refPopup.offsetHeight || 0;
    if (x + width > window.innerWidth - 8) x = Math.max(8, window.innerWidth - width - 8);
    if (y + height > window.innerHeight - 8) y = Math.max(8, window.innerHeight - height - 8);
    refPopup.style.left = x + 'px';
    refPopup.style.top = y + 'px';
  }

  function hideReferencePopup() {
    if (refPopup) refPopup.hidden = true;
  }

  function currentJsonPath() {
    if (Array.isArray(state.selectedPath)) return state.selectedPath;
    if (state.selectedNode) return findJsonPath(state.ast, state.selectedNode, []);
    return null;
  }

  function updateNavButtons() {
    if (backBtn) backBtn.disabled = state.historyIndex <= 0;
    if (forwardBtn) {
      forwardBtn.disabled = state.historyIndex < 0
        || state.historyIndex >= state.history.length - 1;
    }
  }

  function recordNavigation(path) {
    if (!Array.isArray(path)) return;
    const key = pathKey(path);
    if (state.historyIndex >= 0 && pathKey(state.history[state.historyIndex]) === key) return;
    state.history = state.history.slice(0, state.historyIndex + 1);
    state.history.push(path.slice());
    state.historyIndex = state.history.length - 1;
    if (state.history.length > 100) {
      state.history.shift();
      state.historyIndex -= 1;
    }
    updateNavButtons();
  }

  function expandPath(path) {
    if (!Array.isArray(path)) return;
    for (let size = 0; size < path.length; size += 1) {
      state.jsonExpanded.add(pathKey(path.slice(0, size)));
    }
    state.jsonExpanded.add(pathKey(path));
  }

  function navigateToPath(path, record) {
    if (!Array.isArray(path)) return false;
    const node = valueAtJsonPath(state.ast, path);
    if (node === undefined || node === null) return false;
    expandPath(path);
    state.selectedPath = path.slice();
    state.selectedNode = node;
    if (record) recordNavigation(path);
    render();
    scrollSelectedRow();
    return true;
  }

  function navigateToReference(node) {
    const path = findJsonPath(state.ast, node, []);
    if (!path) {
      setTransientStatus('Reference target is not in this tree');
      return;
    }
    const current = currentJsonPath();
    if (current) recordNavigation(current);
    navigateToPath(path, true);
    setTransientStatus('Navigated to ' + (node.kind || 'node'));
  }

  function historyGo(delta) {
    const next = state.historyIndex + delta;
    if (next < 0 || next >= state.history.length) return;
    state.historyIndex = next;
    navigateToPath(state.history[next], false);
    updateNavButtons();
  }

  function loadFavorites() {
    try {
      const saved = vscode.getState ? vscode.getState() : null;
      if (saved && Array.isArray(saved.favorites)) {
        state.favorites = saved.favorites.filter(function (favorite) {
          return favorite && Array.isArray(favorite.path);
        });
      }
    } catch (error) {
      state.favorites = [];
    }
  }

  function persistFavorites() {
    try {
      if (vscode.setState) vscode.setState({ favorites: state.favorites });
    } catch (error) {
      // State persistence is best effort.
    }
  }

  function renderFavorites() {
    if (!favoritesEl) return;
    favoritesEl.textContent = '';
    const placeholder = document.createElement('option');
    placeholder.value = '';
    placeholder.textContent = 'Favorites…';
    favoritesEl.appendChild(placeholder);
    state.favorites.forEach(function (favorite, index) {
      const option = document.createElement('option');
      option.value = String(index);
      option.textContent = favorite.label || pathKey(favorite.path);
      favoritesEl.appendChild(option);
    });
  }

  function toggleFavorite() {
    const path = currentJsonPath();
    if (!path) {
      setTransientStatus('Select a node first');
      return;
    }
    const key = pathKey(path);
    const index = state.favorites.findIndex(function (favorite) {
      return pathKey(favorite.path) === key;
    });
    if (index === -1) {
      const node = valueAtJsonPath(state.ast, path);
      state.favorites.push({ path: path.slice(), label: summaryOf(node) || key });
      setTransientStatus('Added to favorites');
    } else {
      state.favorites.splice(index, 1);
      setTransientStatus('Removed from favorites');
    }
    persistFavorites();
    renderFavorites();
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

  function selectNode(node, path) {
    if (!node) return;
    state.selectedNode = node;
    state.selectedPath = Array.isArray(path) ? path.slice() : null;
    if (state.selectedPath) recordNavigation(state.selectedPath);
    refreshSelection();
    const scrollTop = treeEl.scrollTop;
    render();
    treeEl.scrollTop = scrollTop;
    scrollSelectedRow();
  }

  function renderJsonValue(value, key, path) {
    if (!jsonMatches(value, key)) return null;
    const isNode = value !== null && typeof value === 'object' && typeof value.kind === 'string';
    const isReference = isNode && isReferenceKind(value.kind);
    const reference = isReference ? resolveReference(value) : null;
    const expandable = jsonIsExpandable(value);
    const expanded = expandable
      && (state.filter.length > 0 || state.jsonExpanded.has(pathKey(path)));
    const wrapper = document.createElement('div');
    const row = document.createElement('div');
    row.className = 'json-row' + (expandable ? ' expandable' : '') + (isNode ? ' ast-node' : '')
      + (isReference ? ' ref-link' : '');
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
    if (isReference) {
      const badge = document.createElement('span');
      badge.className = 'ref-badge';
      badge.textContent = '↗';
      badge.title = 'Go to the referenced node';
      row.appendChild(badge);
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
      let text = orderedJson(value);
      if (text.length > 220) text = text.slice(0, 220) + '…';
      preview.textContent = text;
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
        if (isReference && reference) {
          navigateToReference(reference);
        } else if (isReference) {
          setTransientStatus('Reference target not found');
        } else if (isNode) {
          selectNode(value, path);
          if (value.loc && Number.isFinite(value.loc.start)) revealSpan(value.loc, false);
        } else {
          toggleJsonPath(path);
        }
      });
    } else if (isNode) {
      row.addEventListener('click', function (event) {
        event.stopPropagation();
        if (isReference && reference) {
          navigateToReference(reference);
        } else if (isReference) {
          setTransientStatus('Reference target not found');
        } else {
          selectNode(value, path);
          if (value.loc && Number.isFinite(value.loc.start)) revealSpan(value.loc, false);
        }
      });
    }
    if (isReference && reference) {
      row.addEventListener('mouseenter', function (event) {
        showReferencePopup(reference, event);
      });
      row.addEventListener('mousemove', function (event) {
        showReferencePopup(reference, event);
      });
      row.addEventListener('mouseleave', hideReferencePopup);
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
    hideReferencePopup();
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
    let file = typeof span.file === 'string' ? span.file : '';
    if (file && state.ast && Array.isArray(state.ast.files)) {
      const entry = state.ast.files.find(function (candidate) {
        return candidate && candidate.id === file;
      });
      if (entry && typeof entry.path === 'string') file = entry.path;
    }
    vscode.postMessage({
      type: 'reveal',
      start: span.start,
      end: Number.isFinite(span.end) ? span.end : span.start,
      file,
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
    if (!node.loc || !Number.isFinite(node.loc.start)) {
      setTransientStatus('Node has no code position');
      return;
    }
    revealSpan(node.loc, true);
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

  if (backBtn) backBtn.addEventListener('click', function () { historyGo(-1); });
  if (forwardBtn) forwardBtn.addEventListener('click', function () { historyGo(1); });
  if (favoriteBtn) favoriteBtn.addEventListener('click', toggleFavorite);
  if (favoritesEl) {
    favoritesEl.addEventListener('change', function () {
      const value = favoritesEl.value;
      favoritesEl.value = '';
      if (value === '') return;
      const index = Number(value);
      if (!Number.isInteger(index) || index < 0 || index >= state.favorites.length) return;
      navigateToPath(state.favorites[index].path, true);
    });
  }
  treeEl.addEventListener('scroll', hideReferencePopup);

  window.addEventListener('message', function (event) {
    const message = event.data;
    if (!message || typeof message.type !== 'string') return;
    if (message.type === 'setAst') {
      state.ast = message.ast || null;
      state.error = null;
      state.loading = false;
      state.fileName = message.fileName || 'Archol';
      state.applicationMode = Boolean(message.applicationMode);
      buildAstIndex(state.ast);
      state.history = [];
      state.historyIndex = -1;
      updateNavButtons();
      state.inApplication = message.inApplication !== false;
      state.commentedOut = Boolean(message.commentedOut);
      state.selectedPath = Array.isArray(message.selectedPath) ? message.selectedPath : null;
      state.selectedNode = state.selectedPath && state.ast
        ? valueAtJsonPath(state.ast, state.selectedPath)
        : null;
      state.jsonExpanded = new Set(['']);
      renderFavorites();
      if (message.revealSelected && Array.isArray(state.selectedPath)) {
        // Collapse everything except the parents of the selected node. The
        // selected node itself stays collapsed; its row is still visible.
        for (let size = 0; size < state.selectedPath.length; size += 1) {
          state.jsonExpanded.add(pathKey(state.selectedPath.slice(0, size)));
        }
      }
      fileEl.textContent = state.fileName;
      fileEl.title = message.uri || state.fileName;
      updateBanner();
      render();
      if (message.revealSelected) {
        requestAnimationFrame(function () { scrollSelectedRow(); });
      }
      return;
    }
    if (message.type === 'loading') {
      hideReferencePopup();
      state.loading = true;
      state.error = null;
      state.inApplication = true;
      state.commentedOut = false;
      updateBanner();
      render();
      return;
    }
    if (message.type === 'error') {
      hideReferencePopup();
      state.error = String(message.message || 'Failed to load the AST.');
      state.loading = false;
      state.inApplication = true;
      state.commentedOut = false;
      updateBanner();
      render();
    }
  });

  loadFavorites();
  renderFavorites();
  updateNavButtons();
  updateBanner();
  updateStatus();
  refreshSelection();
  vscode.postMessage({ type: 'ready' });
})();`;

module.exports = { browserScript };
