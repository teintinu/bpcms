'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { buildServerCandidates } = require('../src/server-path');

test('resolves the default archol on PATH and then release/debug', () => {
  const candidates = buildServerCandidates('archol', ['/workspace'], 'linux');
  assert.deepEqual(candidates, [
    'archol',
    path.join('/workspace', 'archol', 'target', 'release', 'archol'),
    path.join('/workspace', 'archol', 'target', 'debug', 'archol'),
  ]);
});

test('on Windows uses archol.exe in the fallback paths', () => {
  const candidates = buildServerCandidates('archol', ['C:\\ws'], 'win32');
  assert.deepEqual(candidates.slice(0, 2), ['archol', 'archol.exe']);
  assert.ok(candidates[2].endsWith('archol.exe'));
  assert.ok(candidates[3].endsWith('archol.exe'));
});

test('a configured absolute path comes before the fallbacks', () => {
  const candidates = buildServerCandidates('/opt/archol/bin/archol', ['/workspace'], 'linux');
  assert.equal(candidates[0], '/opt/archol/bin/archol');
  assert.equal(candidates[1], path.join('/workspace', 'archol', 'target', 'release', 'archol'));
});

test('a relative path with a separator is resolved at the workspace root', () => {
  const candidates = buildServerCandidates('bin/archol', ['/workspace'], 'linux');
  assert.equal(candidates[0], path.join('/workspace', 'bin', 'archol'));
  assert.ok(candidates.includes('bin/archol'));
});

test('does not duplicate candidates', () => {
  const candidates = buildServerCandidates('archol', ['/a', '/a'], 'linux');
  assert.equal(new Set(candidates).size, candidates.length);
});
