'use strict';

const vscode = require('vscode');
const { LspClient } = require('./lsp-client');
const { buildServerCandidates } = require('./server-path');
const { applyDiagnostics, showServerMessage } = require('./diagnostics');
const {
  announceStartError,
  applicationSetting,
  configurationValue,
  serverSettings,
  workspaceFolderList,
} = require('./session-support');
const { normalizeAstResult } = require('./app-support');
const { LANGUAGE_ID, isArcholDocument } = require('./vscode-utils');
const CLIENT_CAPABILITIES = {
  textDocument: {
    hover: { contentFormat: ['markdown', 'plaintext'] },
    completion: {
      completionItem: {
        snippetSupport: true,
        documentationFormat: ['markdown', 'plaintext'],
      },
    },
    synchronization: { didSave: true },
    documentSymbol: { hierarchicalDocumentSymbolSupport: true },
    definition: { linkSupport: true },
  },
  workspace: { workspaceFolders: true },
};

class ArcholSession {
  constructor(options) {
    this._diagnostics = options.diagnostics;
    this._setState = options.setState;
    this._log = options.log;
    this._showOutput = options.showOutput;
    this._getAstExplorer = options.getAstExplorer;
    this._extensionVersion = options.extensionVersion;
    this._client = null;
    this._serverReady = false;
    this._starting = null;
    this._openedUris = new Set();
    this._state = 'stopped';
    this._generation = 0;
    this._syncKind = 1;
    this._applicationInFlight = null;
    this._appliedApplication = null;
  }

  get state() {
    return this._state;
  }

  activeClient() {
    return this._serverReady ? this._client : null;
  }

  async ensureClient() {
    if (this._serverReady && this._client) return this._client;
    if (this._starting) return this._starting;
    const started = this._startServer();
    this._starting = started;
    const clear = () => {
      if (this._starting === started) this._starting = null;
    };
    started.then(clear, clear);
    return started;
  }

  async stop() {
    this._generation += 1;
    this._starting = null;
    const instance = this._client;
    this._client = null;
    this._serverReady = false;
    this._applicationInFlight = null;
    this._appliedApplication = null;
    this._openedUris.clear();
    this._diagnostics.clear();
    this._setState('stopped');
    this._state = 'stopped';
    const astExplorer = this._getAstExplorer();
    if (astExplorer) astExplorer.setOffline();
    if (instance) {
      try {
        await instance.stop();
      } catch (error) {
        this._log(`failed to stop server: ${error.message}`);
      }
    }
  }

  sendDidOpen(document) {
    const uri = document.uri.toString();
    if (!this.activeClient() || this._openedUris.has(uri)) return;
    this._openedUris.add(uri);
    this._client.notify('textDocument/didOpen', {
      textDocument: {
        uri,
        languageId: LANGUAGE_ID,
        version: document.version,
        text: document.getText(),
      },
    });
  }

  openDocument(document) {
    if (!isArcholDocument(document) || this._state === 'error') return;
    this.ensureClient()
      .then((instance) => {
        if (instance) this.sendDidOpen(document);
      })
      .catch(() => {
        // The error message was already shown by announceStartError.
      });
  }

  changeDocument(event) {
    const document = event?.document;
    if (!isArcholDocument(document)) return;
    const uri = document.uri.toString();
    if (!this.activeClient()) {
      if (this._state !== 'error') this.openDocument(document);
      return;
    }
    if (!this._openedUris.has(uri)) {
      this.sendDidOpen(document);
      return;
    }
    const contentChanges = this._syncKind === 2
      ? (event.contentChanges || []).map((change) => ({
        range: {
          start: { line: change.range.start.line, character: change.range.start.character },
          end: { line: change.range.end.line, character: change.range.end.character },
        },
        rangeLength: change.rangeLength,
        text: change.text,
      }))
      : [{ text: document.getText() }];
    if (contentChanges.length === 0) return;
    this._client.notify('textDocument/didChange', {
      textDocument: { uri, version: document.version },
      contentChanges,
    });
    const astExplorer = this._getAstExplorer();
    if (astExplorer) astExplorer.scheduleRefresh(uri);
  }

  saveDocument(document) {
    if (!isArcholDocument(document) || !this.activeClient()) return;
    this._client.notify('textDocument/didSave', {
      textDocument: { uri: document.uri.toString() },
      text: document.getText(),
    });
  }

  closeDocument(document) {
    if (!isArcholDocument(document) || !this.activeClient()) return;
    const uri = document.uri.toString();
    if (!this._openedUris.has(uri)) return;
    this._openedUris.delete(uri);
    this._client.notify('textDocument/didClose', { textDocument: { uri } });
  }

  async fetchApplicationAst() {
    const instance = await this.ensureClient();
    const result = await instance.request('archol/applicationAst', { noBody: true });
    return normalizeAstResult(result);
  }

  async fetchDeclaration(id) {
    const instance = await this.ensureClient();
    return instance.request('archol/declaration', { id });
  }

  async fetchUsages(id) {
    const instance = await this.ensureClient();
    return instance.request('archol/usages', { id });
  }

  // Sets the application on the running server. Repeated calls for the same
  // URI share the in-flight request, so the command and the settings listener
  // do not duplicate it.
  setApplication(uri) {
    const key = typeof uri === 'string' ? uri.trim() : '';
    if (this._applicationInFlight && this._applicationInFlight.key === key) {
      return this._applicationInFlight.promise;
    }
    const promise = this.ensureClient()
      .then((instance) => instance.request('archol/setApplication', { uri: key }))
      .then((result) => {
        this._appliedApplication = key;
        const astExplorer = this._getAstExplorer();
        if (astExplorer && astExplorer.visible) {
          astExplorer.refresh().catch((error) => this._log(`failed to refresh AST: ${error.message}`));
        }
        return result;
      })
      .finally(() => {
        if (this._applicationInFlight && this._applicationInFlight.promise === promise) {
          this._applicationInFlight = null;
        }
      });
    this._applicationInFlight = { key, promise };
    return promise;
  }

  // Used when `archol.application` changes outside the select command: notify
  // the running server, or leave it to initializationOptions when stopped.
  async applyConfiguredApplication() {
    if (!this.activeClient()) return null;
    const uri = applicationSetting();
    if (this._appliedApplication === uri) return null;
    return this.setApplication(uri);
  }

  async _startServer() {
    if (this._client) await this.stop();
    const generation = ++this._generation;
    this._diagnostics.clear();
    this._setState('starting');
    this._state = 'starting';

    const roots = (vscode.workspace.workspaceFolders || []).map((folder) => folder.uri.fsPath);
    const { path: configured, args } = serverSettings();
    const candidates = buildServerCandidates(configured, roots);
    this._log(`looking for server: ${candidates.join(' | ')}`);

    const instance = new LspClient({ candidates, cwd: roots[0], args, log: this._log });
    this._wireClient(instance);

    try {
      const command = await instance.start();
      if (generation !== this._generation) {
        await instance.stop();
        throw new Error('Initialization canceled by a restart.');
      }
      this._client = instance;
      this._log(`server running: ${command}`);
      const application = applicationSetting();
      const result = await instance.request('initialize', {
        processId: process.pid,
        clientInfo: { name: 'archol-vscode', version: this._extensionVersion },
        rootUri: roots[0] ? vscode.Uri.file(roots[0]).toString() : null,
        workspaceFolders: workspaceFolderList(),
        capabilities: CLIENT_CAPABILITIES,
        initializationOptions: { application },
      });
      if (generation !== this._generation) {
        await instance.stop();
        throw new Error('Initialization canceled by a restart.');
      }
      const sync = result?.capabilities?.textDocumentSync;
      this._syncKind = typeof sync === 'number' ? sync : Number(sync?.change) || 1;
      instance.notify('initialized', {});
      this._serverReady = true;
      this._state = 'ready';
      this._appliedApplication = application;
      this._setState('ready');
      const serverName = result?.serverInfo?.name || 'archol';
      const serverVersion = result?.serverInfo?.version || '';
      this._log(`server ready: ${`${serverName} ${serverVersion}`.trim()}`);

      for (const document of vscode.workspace.textDocuments) {
        if (isArcholDocument(document)) this.sendDidOpen(document);
      }
      const astExplorer = this._getAstExplorer();
      if (astExplorer && astExplorer.visible) {
        astExplorer.refresh().catch((error) => this._log(`failed to refresh AST: ${error.message}`));
      }
      return instance;
    } catch (error) {
      const current = generation === this._generation;
      this._serverReady = false;
      if (this._client === instance) this._client = null;
      try {
        await instance.stop();
      } catch {
        // The process may already have exited.
      }
      if (current) {
        this._state = 'error';
        this._setState('error');
        this._log(`failed to start server: ${error.stack || error.message}`);
        announceStartError(error, candidates, this._showOutput);
      } else {
        this._log(`previous startup canceled: ${error.message}`);
      }
      const wrapped = new Error(`Archol LSP server unavailable: ${error.message}`);
      wrapped.archolShown = true;
      wrapped.cause = error;
      throw wrapped;
    }
  }
  _wireClient(instance) {
    instance.on('notification', (method, params) => {
      if (method === 'textDocument/publishDiagnostics') {
        try {
          applyDiagnostics(this._diagnostics, params);
        } catch (error) {
          this._log(`invalid diagnostic ignored: ${error.message}`);
        }
      } else if (method === 'window/showMessage') {
        showServerMessage(params, this._log);
      }
    });

    instance.onRequest('workspace/configuration', (params) => {
      const items = Array.isArray(params?.items) ? params.items : [];
      return items.map((item) => configurationValue(item?.section));
    });
    instance.onRequest('workspace/workspaceFolders', () => workspaceFolderList());
    instance.onRequest('window/showMessage', (params) => {
      showServerMessage(params, this._log);
      return null;
    });
    instance.onRequest('window/showMessageRequest', async (params) => {
      if (!params || !params.message) return null;
      const actions = Array.isArray(params.actions) ? params.actions.map((action) => action.title) : [];
      const message = `Archol: ${params.message}`;
      const show = Number(params.type) === 1
        ? vscode.window.showErrorMessage
        : Number(params.type) === 2
          ? vscode.window.showWarningMessage
          : vscode.window.showInformationMessage;
      const picked = actions.length > 0 ? await show(message, ...actions) : await show(message);
      if (!picked) return null;
      const index = actions.indexOf(picked);
      return index >= 0 ? { title: actions[index] } : { title: picked };
    });

    instance.on('stopped', (info) => {
      if (this._client !== instance) return;
      const wasReady = this._serverReady;
      this._serverReady = false;
      this._client = null;
      this._applicationInFlight = null;
      this._appliedApplication = null;
      this._openedUris.clear();
      this._diagnostics.clear();
      if (info.intentional) {
        this._state = 'stopped';
        this._setState('stopped');
        return;
      }
      const reason = info.signal ? `signal ${info.signal}` : `code ${info.code}`;
      this._log(`server exited unexpectedly (${reason})`);
      if (!wasReady) return;
      this._state = 'error';
      this._setState('error');
      vscode.window
        .showWarningMessage(`Archol LSP server exited (${reason}).`, 'Restart')
        .then((action) => {
          if (action === 'Restart') vscode.commands.executeCommand('archol.restartServer');
        });
    });
  }
}

module.exports = { ArcholSession, CLIENT_CAPABILITIES };
