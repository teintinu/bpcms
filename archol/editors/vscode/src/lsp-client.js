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
    this._decoder = new MessageDecoder({ onError: (error) => this.log(`[protocolo] ${error.message}`) });
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
        this.log(`processo iniciado: ${candidate}`);
        this.emit('started', { command: candidate });
        return candidate;
      } catch (error) {
        lastError = error;
        this.log(`não foi possível executar ${candidate}: ${error.message}`);
      }
    }
    const error = new Error(
      `Executável do servidor Archol não encontrado. Tentei: ${this.candidates.join(', ')}`,
    );
    error.code = 'ENOENT';
    error.candidates = this.candidates.slice();
    error.cause = lastError;
    throw error;
  }

  async stop() {
    const child = this._child;
    this._stopping = true;
    this._rejectPending(new Error('Cliente LSP encerrado.'));
    if (!child) {
      this._child = null;
      return;
    }
    try {
      child.stdin.end();
    } catch {
      // O processo pode já ter fechado a entrada.
    }
    if (child.exitCode !== null) {
      this._child = null;
      return;
    }
    try {
      child.kill();
    } catch {
      // Ignorado: o processo pode ter encerrado entre a checagem e o kill.
    }
    await new Promise((resolve) => {
      const timer = setTimeout(() => {
        try {
          child.kill('SIGKILL');
        } catch {
          // Ignorado.
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
      return Promise.reject(new Error('Servidor LSP do Archol não está em execução.'));
    }
    const id = ++this._nextId;
    const timeoutMs = options.timeoutMs || this.requestTimeoutMs;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this._pending.delete(id);
        reject(new Error(`Tempo esgotado aguardando resposta de ${method}.`));
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
      this.log(`falha ao enviar ${method}: ${error.message}`);
    }
  }

  respond(id, result) {
    if (!this._child) return;
    try {
      this._write({ jsonrpc: '2.0', id, result });
      this.log(`← resposta do cliente (id ${id})`);
    } catch (error) {
      this.log(`falha ao responder ${id}: ${error.message}`);
    }
  }

  respondError(id, code, message) {
    if (!this._child) return;
    try {
      this._write({ jsonrpc: '2.0', id, error: { code, message } });
      this.log(`← erro do cliente (id ${id}): ${message}`);
    } catch (error) {
      this.log(`falha ao responder erro ${id}: ${error.message}`);
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
      failureReason: 'O cliente Archol não aplica edições do servidor.',
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
        if (line.trim()) this.log(`[servidor] ${line}`);
      }
    });
    child.stderr.on('error', (error) => this.log(`[stderr] ${error.message}`));
    child.stdin.on('error', (error) => this.log(`[stdin] ${error.message}`));
    child.on('error', (error) => this.log(`[processo] ${error.message}`));
    child.once('exit', (code, signal) => this._handleExit(code, signal));
  }

  _handleExit(code, signal) {
    const intentional = this._stopping;
    this._child = null;
    this._stopping = false;
    const reason = signal ? `sinal ${signal}` : `código ${code}`;
    this.log(`processo encerrado (${reason})`);
    this._rejectPending(new Error(`Servidor LSP do Archol encerrado (${reason}).`));
    this.emit('stopped', { code, signal, intentional });
  }

  _handleMessage(message) {
    if (!message || typeof message !== 'object') return;
    if (typeof message.method === 'string') {
      if (message.id !== undefined && message.id !== null) {
        this._handleServerRequest(message);
      } else {
        if (message.method === 'window/logMessage') {
          this.log(`[servidor] ${message.params?.message || ''}`);
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
      this.log(`requisição do servidor não suportada: ${message.method}`);
      this.respondError(message.id, -32601, `Método não suportado pelo cliente Archol: ${message.method}`);
      return;
    }
    this.log(`← ${message.method} (id ${message.id})`);
    Promise.resolve()
      .then(() => handler(message.params, message))
      .then((result) => this.respond(message.id, result === undefined ? null : result))
      .catch((error) => {
        this.log(`erro ao tratar ${message.method}: ${error.message}`);
        this.respondError(message.id, -32603, error.message);
      });
  }

  _handleResponse(message) {
    const pending = this._pending.get(message.id);
    if (!pending) {
      this.log(`resposta inesperada (id ${message.id})`);
      return;
    }
    this._pending.delete(message.id);
    clearTimeout(pending.timer);
    if (message.error) {
      const error = new Error(`${pending.method}: ${message.error.message || 'erro do servidor'}`);
      error.code = message.error.code;
      error.data = message.error.data;
      this.log(`← ${pending.method} (id ${message.id}) erro ${error.code}: ${message.error.message}`);
      pending.reject(error);
      return;
    }
    this.log(`← ${pending.method} (id ${message.id}) ok`);
    pending.resolve(message.result);
  }

  _write(message) {
    if (!this._child || !this._child.stdin.writable) {
      throw new Error('Servidor LSP do Archol não está em execução.');
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
