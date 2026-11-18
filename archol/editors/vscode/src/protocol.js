'use strict';

// Framing JSON-RPC 2.0 do LSP: cabeçalhos ASCII + corpo UTF-8 medido em bytes.
// Este módulo é puro (não importa `vscode`) e é coberto por test/protocol.test.js.

const CRLF = Buffer.from('\r\n\r\n', 'ascii');
const LF = Buffer.from('\n\n', 'ascii');

function encodeMessage(message) {
  const body = Buffer.from(JSON.stringify(message), 'utf8');
  const header = Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, 'ascii');
  return Buffer.concat([header, body]);
}

function parseContentLength(headerText) {
  const match = /^content-length\s*:\s*(\d+)\s*$/im.exec(headerText);
  if (!match) return null;
  const value = Number(match[1]);
  return Number.isSafeInteger(value) && value >= 0 ? value : null;
}

function parseHeaders(headerText) {
  const headers = new Map();
  for (const line of headerText.split(/\r?\n/)) {
    const separator = line.indexOf(':');
    if (separator <= 0) continue;
    const name = line.slice(0, separator).trim().toLowerCase();
    const value = line.slice(separator + 1).trim();
    if (name) headers.set(name, value);
  }
  return headers;
}

function findHeaderEnd(buffer) {
  const crlf = buffer.indexOf(CRLF);
  const lf = buffer.indexOf(LF);
  if (crlf !== -1) return { index: crlf, separatorLength: CRLF.length };
  if (lf !== -1) return { index: lf, separatorLength: LF.length };
  return null;
}

class MessageDecoder {
  constructor(options = {}) {
    this.errors = [];
    this._onError = typeof options.onError === 'function' ? options.onError : null;
    this._buffer = Buffer.alloc(0);
  }

  get bufferedBytes() {
    return this._buffer.length;
  }

  reset() {
    this._buffer = Buffer.alloc(0);
    this.errors = [];
  }

  append(chunk) {
    if (chunk === null || chunk === undefined) return [];
    const data = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk), 'utf8');
    if (data.length > 0) {
      this._buffer = this._buffer.length > 0
        ? Buffer.concat([this._buffer, data])
        : data;
    }

    const messages = [];
    while (true) {
      const found = findHeaderEnd(this._buffer);
      if (!found) break;

      const headerText = this._buffer.subarray(0, found.index).toString('ascii');
      const contentLength = parseContentLength(headerText);
      if (contentLength === null) {
        this._reportError(new Error(`Cabeçalho LSP sem Content-Length válido: ${headerText.slice(0, 80)}`));
        this._buffer = this._buffer.subarray(found.index + found.separatorLength);
        continue;
      }

      const bodyStart = found.index + found.separatorLength;
      const bodyEnd = bodyStart + contentLength;
      if (this._buffer.length < bodyEnd) break;

      const bodyText = this._buffer.subarray(bodyStart, bodyEnd).toString('utf8');
      this._buffer = this._buffer.subarray(bodyEnd);
      try {
        messages.push(JSON.parse(bodyText));
      } catch (error) {
        this._reportError(new Error(`Corpo LSP não é JSON válido: ${error.message}`));
      }
    }
    return messages;
  }

  _reportError(error) {
    this.errors.push(error);
    if (this._onError) this._onError(error);
  }
}

function decodeMessages(buffer, options = {}) {
  return new MessageDecoder(options).append(buffer);
}

module.exports = {
  MessageDecoder,
  decodeMessages,
  encodeMessage,
  parseContentLength,
  parseHeaders,
};
