'use strict';

const path = require('node:path');
const crypto = require('node:crypto');
const vscode = require('vscode');
const { buildAstHtml } = require('./ast-view-html');
const { normalizeAstResult, usageQuickPickItem } = require('./app-support');
const { byteOffsetAtText, findNodePathAtOffset, textOffsetsForByteSpan } = require('./ast-tree');

const VIEW_TYPE = 'archol.astExplorer';
const REFRESH_DEBOUNCE_MS = 400;

class AstExplorer {
  constructor(options) {
    this._getAst = options.getAst;
    this._getDeclaration = options.getDeclaration;
    this._getUsages = options.getUsages;
    this._log = options.log || (() => {});
    this._showError = options.showError || ((message) => console.error(message));
    this._showInfo = options.showInfo || ((message) => console.info(message));
    this._panel = null;
    this._subscriptions = [];
    this._webviewReady = false;
    this._target = null;
    this._current = null;
    this._selectionPath = null;
    this._selectionOffset = null;
    this._requestSeq = 0;
    this._timer = null;
    this._dirty = false;
    this._disposed = false;
  }

  get visible() {
    return Boolean(this._panel && this._panel.visible);
  }

  async show(target) {
    const uri = target?.uri;
    if (!uri) throw new Error('No document provided for the AST explorer.');
    this._ensurePanel();
    this._panel.reveal(undefined, false);
    this._target = { uri, line: Number(target.line) || 0, character: Number(target.character) || 0 };
    this._selectionPath = null;
    try {
      const document = await vscode.workspace.openTextDocument(vscode.Uri.parse(uri));
      this._selectionOffset = byteOffsetAtText(
        document.getText(),
        this._target.line,
        this._target.character,
      );
    } catch (error) {
      this._log(`could not open ${uri}: ${error.message}`);
      this._selectionOffset = 0;
    }
    await this.refresh({ selectOffset: this._selectionOffset });
  }

  scheduleRefresh(uri) {
    if (this._disposed || !this._panel) return;
    const currentUri = this._current?.uri || this._target?.uri;
    if (currentUri && uri && currentUri !== uri) return;
    if (!this._panel.visible) {
      this._dirty = true;
      return;
    }
    if (this._timer) clearTimeout(this._timer);
    this._timer = setTimeout(() => {
      this._timer = null;
      this.refresh().catch((error) => this._log(`failed to refresh AST: ${error.message}`));
    }, REFRESH_DEBOUNCE_MS);
  }

  async refresh(options = {}) {
    if (this._disposed || !this._panel || !this._target) return;
    const requestId = ++this._requestSeq;
    const uri = this._target.uri;
    if (!this._current) this._post({ type: 'loading' });
    try {
      const result = normalizeAstResult(await this._getAst(uri));
      if (requestId !== this._requestSeq || this._disposed || !this._panel) return;
      const ast = result.ast;
      if (!ast) throw new Error('The server did not return an AST for this document.');
      const document = await vscode.workspace.openTextDocument(vscode.Uri.parse(uri));
      const explicitOffset = options.selectOffset !== undefined && options.selectOffset !== null;
      let offset = explicitOffset ? options.selectOffset : this._selectionOffset;
      let selectedPath = null;
      if (Number.isFinite(offset)) selectedPath = findNodePathAtOffset(ast, offset);
      if (!selectedPath && !explicitOffset && this._selectionPath) selectedPath = this._selectionPath;
      this._selectionOffset = Number.isFinite(offset) ? offset : null;
      this._selectionPath = selectedPath;
      this._current = {
        ast,
        uri,
        fileName: path.basename(document.uri.fsPath) || 'Archol',
        inApplication: result.inApplication,
        commentedOut: result.commentedOut,
      };
      this._dirty = false;
      this._post({
        type: 'setAst',
        ast,
        uri,
        fileName: this._current.fileName,
        selectedPath,
        offset: this._selectionOffset,
        inApplication: this._current.inApplication,
        commentedOut: this._current.commentedOut,
      });
    } catch (error) {
      if (requestId !== this._requestSeq || this._disposed) return;
      this._log(`failed to load AST from ${uri}: ${error.message}`);
      this._current = null;
      this._post({ type: 'error', message: error.message });
      if (!error.archolShown) {
        this._showError(`could not load the AST: ${error.message}`);
      }
    }
  }

  setOffline() {
    this._requestSeq += 1;
    this._current = null;
    if (this._panel) {
      this._post({ type: 'error', message: 'Archol LSP server is not running.' });
    }
  }

  dispose() {
    if (this._disposed) return;
    this._disposed = true;
    this._requestSeq += 1;
    if (this._timer) {
      clearTimeout(this._timer);
      this._timer = null;
    }
    for (const subscription of this._subscriptions) subscription.dispose();
    this._subscriptions = [];
    const panel = this._panel;
    this._panel = null;
    this._webviewReady = false;
    if (panel) panel.dispose();
  }

  _ensurePanel() {
    if (this._panel) return;
    this._panel = vscode.window.createWebviewPanel(
      VIEW_TYPE,
      'Archol: AST',
      { viewColumn: vscode.ViewColumn.Beside, preserveFocus: false },
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [],
      },
    );
    this._webviewReady = false;
    this._current = null;
    const nonce = crypto.randomBytes(16).toString('hex');
    this._panel.webview.html = buildAstHtml({
      cspSource: this._panel.webview.cspSource,
      nonce,
    });
    this._subscriptions.push(
      this._panel.webview.onDidReceiveMessage((message) => this._onMessage(message)),
      this._panel.onDidDispose(() => this._onPanelDisposed()),
      this._panel.onDidChangeViewState((event) => {
        if (event.webviewPanel.visible && this._dirty) {
          this.refresh().catch((error) => this._log(`failed to refresh AST: ${error.message}`));
        }
      }),
    );
  }

  _onPanelDisposed() {
    for (const subscription of this._subscriptions) subscription.dispose();
    this._subscriptions = [];
    this._panel = null;
    this._webviewReady = false;
    this._current = null;
    this._dirty = false;
    if (this._timer) {
      clearTimeout(this._timer);
      this._timer = null;
    }
  }

  _onMessage(message) {
    if (!message || typeof message.type !== 'string') return;
    if (message.type === 'ready') {
      this._webviewReady = true;
      this._syncWebview();
      return;
    }
    if (message.type === 'reveal') {
      const start = Number.isFinite(Number(message.start))
        ? Number(message.start)
        : Number(message.offset);
      const end = Number.isFinite(Number(message.end)) ? Number(message.end) : start;
      if (Array.isArray(message.path)) this._selectionPath = message.path;
      if (Number.isFinite(start)) this._selectionOffset = start;
      this._reveal(start, end, Boolean(message.focus)).catch((error) => {
        this._log(`failed to go to code: ${error.message}`);
      });
      return;
    }
    if (message.type === 'declaration') {
      this._goToDeclaration(message.id).catch((error) => {
        this._reportNavigationError('declaration', error);
      });
      return;
    }
    if (message.type === 'usages') {
      this._showUsages(message.id).catch((error) => {
        this._reportNavigationError('usages', error);
      });
    }
  }

  _reportNavigationError(action, error) {
    this._log(`failed to get ${action}: ${error.message}`);
    this._showError(`could not get ${action}: ${error.message}`);
  }

  async _goToDeclaration(id) {
    if (id === undefined || id === null) return;
    const location = await this._getDeclaration(id);
    if (!location || !location.uri || !location.range) {
      this._showInfo('No declaration found');
      return;
    }
    await this._revealLocation(location);
  }

  async _showUsages(id) {
    if (id === undefined || id === null) return;
    const result = await this._getUsages(id);
    const locations = Array.isArray(result?.locations) ? result.locations : [];
    if (locations.length === 0) {
      this._showInfo('No usages found');
      return;
    }
    const items = locations.map((location) => {
      let rootUri = '';
      try {
        rootUri = vscode.workspace.getWorkspaceFolder(vscode.Uri.parse(location.uri))?.uri.toString() || '';
      } catch {
        // Keeps the absolute path as the label.
      }
      return usageQuickPickItem(location, rootUri);
    });
    const picked = await vscode.window.showQuickPick(items, {
      placeHolder: `${locations.length} usage${locations.length === 1 ? '' : 's'} — select to open`,
      matchOnDescription: true,
    });
    if (!picked) return;
    await this._revealLocation(picked.location);
  }

  _syncWebview() {
    if (!this._webviewReady) return;
    if (this._current) {
      this._post({
        type: 'setAst',
        ast: this._current.ast,
        uri: this._current.uri,
        fileName: this._current.fileName,
        selectedPath: this._selectionPath,
        offset: this._selectionOffset,
        inApplication: this._current.inApplication,
        commentedOut: this._current.commentedOut,
      });
    } else {
      this._post({ type: 'loading' });
    }
  }

  async _reveal(startByte, endByte, focus) {
    if (!this._current) return;
    const uri = vscode.Uri.parse(this._current.uri);
    const document = await vscode.workspace.openTextDocument(uri);
    const text = document.getText();
    const textLength = Buffer.byteLength(text, 'utf8');
    const start = Math.max(0, Math.min(Number(startByte) || 0, textLength));
    const end = Math.max(start, Math.min(Number(endByte) || start, textLength));
    const [startOffset, endOffset] = textOffsetsForByteSpan(text, start, end);
    const startPosition = document.positionAt(startOffset);
    const endPosition = document.positionAt(endOffset);
    await this._showAndSelect(document, startPosition, endPosition, focus);
  }

  // Selects an LSP range returned by `archol/declaration` or `archol/usages`.
  async _revealLocation(location) {
    const uri = vscode.Uri.parse(String(location.uri));
    const document = await vscode.workspace.openTextDocument(uri);
    const range = location.range || {};
    const startLine = Math.max(0, Math.trunc(Number(range.start?.line) || 0));
    const startCharacter = Math.max(0, Math.trunc(Number(range.start?.character) || 0));
    const endLine = Math.max(0, Math.trunc(Number(range.end?.line) || 0));
    const endCharacter = Math.max(0, Math.trunc(Number(range.end?.character) || 0));
    const startPosition = new vscode.Position(startLine, startCharacter);
    const endPosition = new vscode.Position(endLine, endCharacter);
    await this._showAndSelect(document, startPosition, endPosition, true);
  }

  async _showAndSelect(document, startPosition, endPosition, focus) {
    const uri = document.uri;
    // Reuses the already open editor (same tab/column) instead of opening another.
    const existing = vscode.window.visibleTextEditors.find(
      (candidate) => candidate.document.uri.toString() === uri.toString(),
    );
    const options = { preview: false, preserveFocus: !focus };
    if (existing) options.viewColumn = existing.viewColumn;
    const editor = await vscode.window.showTextDocument(document, options);
    const selection = new vscode.Selection(startPosition, endPosition);
    editor.selection = selection;
    editor.revealRange(
      new vscode.Range(startPosition, endPosition),
      vscode.TextEditorRevealType.InCenterIfOutsideViewport,
    );
  }

  _post(message) {
    if (!this._panel || !this._webviewReady) return;
    this._panel.webview.postMessage(message).then(undefined, (error) => {
      this._log(`failed to send message to webview: ${error.message}`);
    });
  }
}

module.exports = { AstExplorer, VIEW_TYPE };
