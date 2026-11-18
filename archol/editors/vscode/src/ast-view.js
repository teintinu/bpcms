'use strict';

const path = require('node:path');
const crypto = require('node:crypto');
const vscode = require('vscode');
const { buildAstHtml } = require('./ast-view-html');
const { byteOffsetAtText, findNodePathAtOffset, textOffsetsForByteSpan } = require('./ast-tree');

const VIEW_TYPE = 'archol.astExplorer';
const REFRESH_DEBOUNCE_MS = 400;

class AstExplorer {
  constructor(options) {
    this._getAst = options.getAst;
    this._log = options.log || (() => {});
    this._showError = options.showError || ((message) => console.error(message));
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
    if (!uri) throw new Error('Nenhum documento informado para o explorador de AST.');
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
      this._log(`não foi possível abrir ${uri}: ${error.message}`);
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
      this.refresh().catch((error) => this._log(`falha ao atualizar AST: ${error.message}`));
    }, REFRESH_DEBOUNCE_MS);
  }

  async refresh(options = {}) {
    if (this._disposed || !this._panel || !this._target) return;
    const requestId = ++this._requestSeq;
    const uri = this._target.uri;
    if (!this._current) this._post({ type: 'loading' });
    try {
      const ast = await this._getAst(uri);
      if (requestId !== this._requestSeq || this._disposed || !this._panel) return;
      if (!ast) throw new Error('O servidor não retornou uma AST para este documento.');
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
      };
      this._dirty = false;
      this._post({
        type: 'setAst',
        ast,
        uri,
        fileName: this._current.fileName,
        selectedPath,
        offset: this._selectionOffset,
      });
    } catch (error) {
      if (requestId !== this._requestSeq || this._disposed) return;
      this._log(`falha ao carregar AST de ${uri}: ${error.message}`);
      this._current = null;
      this._post({ type: 'error', message: error.message });
      if (!error.archolShown) {
        this._showError(`não foi possível carregar a AST: ${error.message}`);
      }
    }
  }

  setOffline() {
    this._requestSeq += 1;
    this._current = null;
    if (this._panel) {
      this._post({ type: 'error', message: 'Servidor LSP do Archol não está em execução.' });
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
          this.refresh().catch((error) => this._log(`falha ao atualizar AST: ${error.message}`));
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
        this._log(`falha ao ir para o código: ${error.message}`);
      });
    }
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

    // Reaproveita o editor já aberto (mesma aba/coluna) em vez de abrir outro.
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
      this._log(`falha ao enviar mensagem para o webview: ${error.message}`);
    });
  }
}

module.exports = { AstExplorer, VIEW_TYPE };
