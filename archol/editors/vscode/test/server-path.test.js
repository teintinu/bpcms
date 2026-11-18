'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { buildServerCandidates } = require('../src/server-path');

test('resolve o padrão archol no PATH e depois release/debug', () => {
  const candidates = buildServerCandidates('archol', ['/workspace'], 'linux');
  assert.deepEqual(candidates, [
    'archol',
    path.join('/workspace', 'archol', 'target', 'release', 'archol'),
    path.join('/workspace', 'archol', 'target', 'debug', 'archol'),
  ]);
});

test('no Windows usa archol.exe nos caminhos de fallback', () => {
  const candidates = buildServerCandidates('archol', ['C:\\ws'], 'win32');
  assert.deepEqual(candidates.slice(0, 2), ['archol', 'archol.exe']);
  assert.ok(candidates[2].endsWith('archol.exe'));
  assert.ok(candidates[3].endsWith('archol.exe'));
});

test('caminho absoluto configurado vem antes dos fallbacks', () => {
  const candidates = buildServerCandidates('/opt/archol/bin/archol', ['/workspace'], 'linux');
  assert.equal(candidates[0], '/opt/archol/bin/archol');
  assert.equal(candidates[1], path.join('/workspace', 'archol', 'target', 'release', 'archol'));
});

test('caminho relativo com separador é resolvido na raiz do workspace', () => {
  const candidates = buildServerCandidates('bin/archol', ['/workspace'], 'linux');
  assert.equal(candidates[0], path.join('/workspace', 'bin', 'archol'));
  assert.ok(candidates.includes('bin/archol'));
});

test('não duplica candidatos', () => {
  const candidates = buildServerCandidates('archol', ['/a', '/a'], 'linux');
  assert.equal(new Set(candidates).size, candidates.length);
});
