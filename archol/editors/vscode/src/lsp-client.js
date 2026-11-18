'use strict';

const { EventEmitter } = require('node:events');
const { MessageDecoder, encodeMessage } = require('./protocol');
const { describeParams } = require('./lsp-log');
const { spawnProcess } = require('./server-path');

const DEFAULT_TIMEOUT_MS = 30000;
const FORCE_KILL_MS = 1500;

class LspClient extends EventEmitter {
  constructor(options = {}) {
    super();
    this.candidates = (options.candidates || [options.command || 'archol']).filter(Boolean);
    this.args = options.args || [];
    this.cwd = options.cwd;
    this.env = options.env || process.env;
    this.log = typeof options.log === 'function' ? options.log : () => {};
    this.requestTimeoutMs = options.requestTimeoutMs || DEFAULT_TIMEOUT_MS;

    this._child = null;
    this._startedCommand = null;
    this._decoder = new MessageDecoder({ onError: (error) => this.log(`[protocol] ${error.message}`) });
    this._nextId = 0;
    this._pending = new Map();
    this._handlers = new Map();
    this._stopping = false;
    this._installDefaultHandlers();
  }

  get running() {
    return Boolean(this._child);
  }

  onRequest(method, handler) {
    this._handlers.set(method, handler);
    return this;
  }

  async start() {
    if (this._child) return this._startedCommand;
    let lastError = null;
    for (const candidate of this.candidates) {
      try {
        const child = await spawnProcess(candidate, this.args, this.cwd, this.env);
        this._child = child;
        this._startedCommand = candidate;
        this._stopping = false;
        this._attach(child);
        this.log(`process started: ${candidate}`);
        this.emit('started', { command: candidate });
        return candidate;
      } catch (error) {
        lastError = error;
        this.log(`could not run ${candidate}: ${error.message}`);
      }
    }
    const error = new Error(
      `Archol server executable not found. Tried: ${this.candidates.join(', ')}`,
    );
    error.code = 'ENOENT';
    error.candidates = this.candidates.slice();
    error.cause = lastError;
    throw error;
  }

  async stop() {
    const child = this._child;
    this._stopping = true;
    this._rejectPending(new Error('LSP client stopped.'));
    if (!child) {
      this._child = null;
      return;
    }
    try {
      child.stdin.end();
    } catch {
      // The process may have already closed stdin.
    }
    if (child.exitCode !== null) {
      this._child = null;
      return;
    }
    try {
      child.kill();
    } catch {
      // Ignored: the process may have exited between the check and the kill.
    }
    await new Promise((resolve) => {
      const timer = setTimeout(() => {
        try {
          child.kill('SIGKILL');
        } catch {
          // Ignored.
        }
        resolve();
      }, FORCE_KILL_MS);
      timer.unref?.();
      child.once('exit', () => {
        clearTimeout(timer);
        resolve();
      });
    });
    this._child = null;
  }

  request(method, params, options = {}) {
    if (!this._child) {
      return Promise.reject(new Error('Archol LSP server is not running.'));
    }
    const id = ++this._nextId;
    const timeoutMs = options.timeoutMs || this.requestTimeoutMs;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this._pending.delete(id);
        reject(new Error(`Timed out waiting for a response from ${method}.`));
      }, timeoutMs);
      timer.unref?.();
      this._pending.set(id, { method, resolve, reject, timer });
      this.log(`→ ${method} (id ${id})${describeParams(params)}`);
      try {
        this._write({ jsonrpc: '2.0', id, method, params });
      } catch (error) {
        clearTimeout(timer);
        this._pending.delete(id);
        reject(error);
      }
    });
  }

  notify(method, params) {
    if (!this._child) return;
    this.log(`→ ${method}${describeParams(params)}`);
    try {
      this._write({ jsonrpc: '2.0', method, params });
    } catch (error) {
      this.log(`failed to send ${method}: ${error.message}`);
    }
  }

  respond(id, result) {
    if (!this._child) return;
    try {
      this._write({ jsonrpc: '2.0', id, result });
      this.log(`← client response (id ${id})`);
    } catch (error) {
      this.log(`failed to respond ${id}: ${error.message}`);
    }
  }

  respondError(id, code, message) {
    if (!this._child) return;
    try {
      this._write({ jsonrpc: '2.0', id, error: { code, message } });
      this.log(`← client error (id ${id}): ${message}`);
    } catch (error) {
      this.log(`failed to respond with error ${id}: ${error.message}`);
    }
  }

  _installDefaultHandlers() {
    this.onRequest('workspace/configuration', (params) => {
      const items = Array.isArray(params?.items) ? params.items : [];
      return items.map(() => null);
    });
    this.onRequest('client/registerCapability', () => null);
    this.onRequest('client/unregisterCapability', () => null);
    this.onRequest('window/workDoneProgress/create', () => null);
    this.onRequest('workspace/workspaceFolders', () => null);
    this.onRequest('workspace/applyEdit', () => ({
      applied: false,
      failureReason: 'The Archol client does not apply server edits.',
    }));
    this.onRequest('window/showMessage', (params) => {
      this.emit('showMessage', params);
      return null;
    });
    this.onRequest('window/showMessageRequest', () => null);
  }

  _attach(child) {
    this._decoder.reset();
    child.stdout.on('data', (chunk) => {
      for (const message of this._decoder.append(chunk)) this._handleMessage(message);
    });
    child.stdout.on('error', (error) => this.log(`[stdout] ${error.message}`));
    child.stderr.on('data', (chunk) => {
      const text = Buffer.isBuffer(chunk) ? chunk.toString('utf8') : String(chunk);
      for (const line of text.split(/\r?\n/)) {
        if (line.trim()) this.log(`[server] ${line}`);
      }
    });
    child.stderr.on('error', (error) => this.log(`[stderr] ${error.message}`));
    child.stdin.on('error', (error) => this.log(`[stdin] ${error.message}`));
    child.on('error', (error) => this.log(`[process] ${error.message}`));
    child.once('exit', (code, signal) => this._handleExit(code, signal));
  }

  _handleExit(code, signal) {
    const intentional = this._stopping;
    this._child = null;
    this._stopping = false;
    const reason = signal ? `signal ${signal}` : `code ${code}`;
    this.log(`process exited (${reason})`);
    this._rejectPending(new Error(`Archol LSP server exited (${reason}).`));
    this.emit('stopped', { code, signal, intentional });
  }

  _handleMessage(message) {
    if (!message || typeof message !== 'object') return;
    if (typeof message.method === 'string') {
      if (message.id !== undefined && message.id !== null) {
        this._handleServerRequest(message);
      } else {
        if (message.method === 'window/logMessage') {
          this.log(`[server] ${message.params?.message || ''}`);
        }
        this.emit('notification', message.method, message.params);
      }
      return;
    }
    if (message.id !== undefined && message.id !== null) {
      this._handleResponse(message);
    }
  }

  _handleServerRequest(message) {
    const handler = this._handlers.get(message.method);
    if (!handler) {
      this.log(`unsupported server request: ${message.method}`);
      this.respondError(message.id, -32601, `Method not supported by the Archol client: ${message.method}`);
      return;
    }
    this.log(`← ${message.method} (id ${message.id})`);
    Promise.resolve()
      .then(() => handler(message.params, message))
      .then((result) => this.respond(message.id, result === undefined ? null : result))
      .catch((error) => {
        this.log(`error handling ${message.method}: ${error.message}`);
        this.respondError(message.id, -32603, error.message);
      });
  }

  _handleResponse(message) {
    const pending = this._pending.get(message.id);
    if (!pending) {
      this.log(`unexpected response (id ${message.id})`);
      return;
    }
    this._pending.delete(message.id);
    clearTimeout(pending.timer);
    if (message.error) {
      const error = new Error(`${pending.method}: ${message.error.message || 'server error'}`);
      error.code = message.error.code;
      error.data = message.error.data;
      this.log(`← ${pending.method} (id ${message.id}) error ${error.code}: ${message.error.message}`);
      pending.reject(error);
      return;
    }
    this.log(`← ${pending.method} (id ${message.id}) ok`);
    pending.resolve(message.result);
  }

  _write(message) {
    if (!this._child || !this._child.stdin.writable) {
      throw new Error('Archol LSP server is not running.');
    }
    this._child.stdin.write(encodeMessage(message));
  }

  _rejectPending(error) {
    for (const pending of this._pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this._pending.clear();
  }
}

module.exports = {
  LspClient,
  buildServerCandidates: require('./server-path').buildServerCandidates,
};
