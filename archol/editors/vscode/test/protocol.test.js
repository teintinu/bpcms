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

test('codifica e decodifica uma mensagem', () => {
  const message = { jsonrpc: '2.0', id: 1, method: 'initialize', params: { rootUri: null } };
  const framed = encodeMessage(message);
  assert.ok(Buffer.isBuffer(framed));
  assert.match(framed.toString('ascii'), /^Content-Length: \d+\r\n\r\n/);
  assert.deepEqual(decodeMessages(framed), [message]);
});

test('Content-Length mede bytes UTF-8, não caracteres', () => {
  const message = { text: 'nota fiscal — ação 😀' };
  const framed = encodeMessage(message);
  const body = Buffer.from(JSON.stringify(message), 'utf8');
  assert.match(framed.toString('ascii'), new RegExp(`Content-Length: ${body.length}\\r\\n\\r\\n`));
  assert.deepEqual(decodeMessages(framed), [message]);
});

test('decodifica múltiplas mensagens no mesmo buffer', () => {
  const messages = [
    { jsonrpc: '2.0', id: 1, result: { ok: true } },
    { jsonrpc: '2.0', method: 'textDocument/publishDiagnostics', params: { uri: 'file:///a.archol' } },
    { jsonrpc: '2.0', id: 2, result: null },
  ];
  const buffer = Buffer.concat(messages.map(rawFrame));
  assert.deepEqual(decodeMessages(buffer), messages);
});

test('aceita mensagens divididas em qualquer fronteira de byte', () => {
  const messages = [
    { jsonrpc: '2.0', id: 7, result: { nome: 'João', emoji: '📦' } },
    { jsonrpc: '2.0', method: 'window/logMessage', params: { message: 'pronto' } },
  ];
  const buffer = Buffer.concat(messages.map(rawFrame));
  for (let split = 0; split <= buffer.length; split += 1) {
    const decoder = new MessageDecoder();
    const received = [];
    received.push(...decoder.append(buffer.subarray(0, split)));
    received.push(...decoder.append(buffer.subarray(split)));
    assert.deepEqual(received, messages, `divisão no byte ${split}`);
    assert.equal(decoder.bufferedBytes, 0);
  }
});

test('aceita buffers divididos byte a byte, inclusive no corpo UTF-8', () => {
  const message = { jsonrpc: '2.0', id: 3, result: { texto: 'coração 💚' } };
  const buffer = rawFrame(message);
  const decoder = new MessageDecoder();
  const received = [];
  for (const byte of buffer) {
    received.push(...decoder.append(Buffer.from([byte])));
  }
  assert.deepEqual(received, [message]);
  assert.equal(decoder.bufferedBytes, 0);
});

test('aceita chunks de string e combina com Buffer', () => {
  const message = { jsonrpc: '2.0', id: 4, result: { valor: 'ok' } };
  const buffer = rawFrame(message);
  const split = buffer.indexOf('\r\n\r\n') + 4 + 5;
  const decoder = new MessageDecoder();
  const first = decoder.append(buffer.subarray(0, split).toString('utf8'));
  const second = decoder.append(buffer.subarray(split));
  assert.deepEqual([...first, ...second], [message]);
});

test('tolera cabeçalhos extras e nome do cabeçalho em minúsculas', () => {
  const body = Buffer.from(JSON.stringify({ jsonrpc: '2.0', id: 5, result: 'ok' }), 'utf8');
  const frame = Buffer.concat([
    Buffer.from(`content-length: ${body.length}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n`, 'ascii'),
    body,
  ]);
  assert.deepEqual(decodeMessages(frame), [{ jsonrpc: '2.0', id: 5, result: 'ok' }]);
});

test('parseContentLength valida o cabeçalho', () => {
  assert.equal(parseContentLength('Content-Length: 42'), 42);
  assert.equal(parseContentLength('content-length: 0'), 0);
  assert.equal(parseContentLength('Content-Type: application/json'), null);
  assert.equal(parseContentLength('Content-Length: abc'), null);
  assert.equal(parseContentLength('Content-Length: -1'), null);
});

test('reporta corpo JSON inválido e continua no próximo frame', () => {
  const errors = [];
  const decoder = new MessageDecoder({ onError: (error) => errors.push(error) });
  const invalidBody = Buffer.from('{não é json', 'utf8');
  const invalidFrame = Buffer.concat([
    Buffer.from(`Content-Length: ${invalidBody.length}\r\n\r\n`, 'ascii'),
    invalidBody,
  ]);
  const valid = { jsonrpc: '2.0', id: 6, result: 'depois do erro' };
  const received = decoder.append(Buffer.concat([invalidFrame, rawFrame(valid)]));
  assert.deepEqual(received, [valid]);
  assert.equal(errors.length, 1);
  assert.match(errors[0].message, /JSON válido/);
});

test('reporta cabeçalho sem Content-Length e não entra em loop', () => {
  const errors = [];
  const decoder = new MessageDecoder({ onError: (error) => errors.push(error) });
  const received = decoder.append(Buffer.from('X-Teste: 1\r\n\r\n{}', 'ascii'));
  assert.deepEqual(received, []);
  assert.equal(errors.length, 1);
  assert.equal(decoder.bufferedBytes, 2); // o "{}" restante não é um frame válido
});

test('reset limpa buffer e erros', () => {
  const decoder = new MessageDecoder();
  decoder.append('Content-Length: 100\r\n\r\nparcial');
  assert.ok(decoder.bufferedBytes > 0);
  decoder.reset();
  assert.equal(decoder.bufferedBytes, 0);
  assert.deepEqual(decoder.errors, []);
});

test('aceita separador LF LF por tolerância', () => {
  const body = JSON.stringify({ jsonrpc: '2.0', id: 8, result: true });
  const frame = Buffer.from(`Content-Length: ${Buffer.byteLength(body)}\n\n${body}`, 'utf8');
  assert.deepEqual(decodeMessages(frame), [{ jsonrpc: '2.0', id: 8, result: true }]);
});
