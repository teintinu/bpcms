'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { MessageDecoder, decodeMessages, encodeMessage, parseContentLength } = require('../src/protocol');

function rawFrame(value) {
  const body = Buffer.from(JSON.stringify(value), 'utf8');
  return Buffer.concat([
    Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, 'ascii'),
    body,
  ]);
}

test('encodes and decodes a message', () => {
  const message = { jsonrpc: '2.0', id: 1, method: 'initialize', params: { rootUri: null } };
  const framed = encodeMessage(message);
  assert.ok(Buffer.isBuffer(framed));
  assert.match(framed.toString('ascii'), /^Content-Length: \d+\r\n\r\n/);
  assert.deepEqual(decodeMessages(framed), [message]);
});

test('Content-Length measures UTF-8 bytes, not characters', () => {
  const message = { text: 'invoice — action 😀' };
  const framed = encodeMessage(message);
  const body = Buffer.from(JSON.stringify(message), 'utf8');
  assert.match(framed.toString('ascii'), new RegExp(`Content-Length: ${body.length}\\r\\n\\r\\n`));
  assert.deepEqual(decodeMessages(framed), [message]);
});

test('decodes multiple messages from the same buffer', () => {
  const messages = [
    { jsonrpc: '2.0', id: 1, result: { ok: true } },
    { jsonrpc: '2.0', method: 'textDocument/publishDiagnostics', params: { uri: 'file:///a.archol' } },
    { jsonrpc: '2.0', id: 2, result: null },
  ];
  const buffer = Buffer.concat(messages.map(rawFrame));
  assert.deepEqual(decodeMessages(buffer), messages);
});

test('accepts messages split at any byte boundary', () => {
  const messages = [
    { jsonrpc: '2.0', id: 7, result: { name: 'João', emoji: '📦' } },
    { jsonrpc: '2.0', method: 'window/logMessage', params: { message: 'ready' } },
  ];
  const buffer = Buffer.concat(messages.map(rawFrame));
  for (let split = 0; split <= buffer.length; split += 1) {
    const decoder = new MessageDecoder();
    const received = [];
    received.push(...decoder.append(buffer.subarray(0, split)));
    received.push(...decoder.append(buffer.subarray(split)));
    assert.deepEqual(received, messages, `split at byte ${split}`);
    assert.equal(decoder.bufferedBytes, 0);
  }
});

test('accepts buffers split byte by byte, including the UTF-8 body', () => {
  const message = { jsonrpc: '2.0', id: 3, result: { text: 'heart 💚' } };
  const buffer = rawFrame(message);
  const decoder = new MessageDecoder();
  const received = [];
  for (const byte of buffer) {
    received.push(...decoder.append(Buffer.from([byte])));
  }
  assert.deepEqual(received, [message]);
  assert.equal(decoder.bufferedBytes, 0);
});

test('accepts string chunks and combines them with Buffers', () => {
  const message = { jsonrpc: '2.0', id: 4, result: { value: 'ok' } };
  const buffer = rawFrame(message);
  const split = buffer.indexOf('\r\n\r\n') + 4 + 5;
  const decoder = new MessageDecoder();
  const first = decoder.append(buffer.subarray(0, split).toString('utf8'));
  const second = decoder.append(buffer.subarray(split));
  assert.deepEqual([...first, ...second], [message]);
});

test('tolerates extra headers and a lowercase header name', () => {
  const body = Buffer.from(JSON.stringify({ jsonrpc: '2.0', id: 5, result: 'ok' }), 'utf8');
  const frame = Buffer.concat([
    Buffer.from(`content-length: ${body.length}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n`, 'ascii'),
    body,
  ]);
  assert.deepEqual(decodeMessages(frame), [{ jsonrpc: '2.0', id: 5, result: 'ok' }]);
});

test('parseContentLength validates the header', () => {
  assert.equal(parseContentLength('Content-Length: 42'), 42);
  assert.equal(parseContentLength('content-length: 0'), 0);
  assert.equal(parseContentLength('Content-Type: application/json'), null);
  assert.equal(parseContentLength('Content-Length: abc'), null);
  assert.equal(parseContentLength('Content-Length: -1'), null);
});

test('reports an invalid JSON body and continues to the next frame', () => {
  const errors = [];
  const decoder = new MessageDecoder({ onError: (error) => errors.push(error) });
  const invalidBody = Buffer.from('{not json', 'utf8');
  const invalidFrame = Buffer.concat([
    Buffer.from(`Content-Length: ${invalidBody.length}\r\n\r\n`, 'ascii'),
    invalidBody,
  ]);
  const valid = { jsonrpc: '2.0', id: 6, result: 'after the error' };
  const received = decoder.append(Buffer.concat([invalidFrame, rawFrame(valid)]));
  assert.deepEqual(received, [valid]);
  assert.equal(errors.length, 1);
  assert.match(errors[0].message, /valid JSON/);
});

test('reports a header without Content-Length and does not loop', () => {
  const errors = [];
  const decoder = new MessageDecoder({ onError: (error) => errors.push(error) });
  const received = decoder.append(Buffer.from('X-Test: 1\r\n\r\n{}', 'ascii'));
  assert.deepEqual(received, []);
  assert.equal(errors.length, 1);
  assert.equal(decoder.bufferedBytes, 2); // the remaining "{}" is not a valid frame
});

test('reset clears the buffer and errors', () => {
  const decoder = new MessageDecoder();
  decoder.append('Content-Length: 100\r\n\r\nparcial');
  assert.ok(decoder.bufferedBytes > 0);
  decoder.reset();
  assert.equal(decoder.bufferedBytes, 0);
  assert.deepEqual(decoder.errors, []);
});

test('accepts an LF LF separator as a tolerance', () => {
  const body = JSON.stringify({ jsonrpc: '2.0', id: 8, result: true });
  const frame = Buffer.from(`Content-Length: ${Buffer.byteLength(body)}\n\n${body}`, 'utf8');
  assert.deepEqual(decodeMessages(frame), [{ jsonrpc: '2.0', id: 8, result: true }]);
});
