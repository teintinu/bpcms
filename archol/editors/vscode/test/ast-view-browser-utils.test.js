'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { browserUtils } = require('../src/ast-view-browser-utils');

const utils = new Function(
  browserUtils + '; return { orderedKeys, orderedJson, orderedJsonPretty, nodeAtPath };',
)();

test('orderedKeys puts kind/title/name/value/message/id first and locs last', () => {
  const value = {
    loc: { start: 1, end: 2 },
    z: 1,
    kind: 'Process',
    keywordLoc: { start: 1, end: 2 },
    id: 'a',
    title: 'T',
    name: 'N',
    value: 3,
    message: 'm',
    a: 2,
  };
  assert.deepEqual(utils.orderedKeys(value), [
    'kind',
    'title',
    'name',
    'value',
    'message',
    'id',
    'z',
    'a',
    'loc',
    'keywordLoc',
  ]);
});

test('collapsed inline JSON follows the same order', () => {
  const text = utils.orderedJson({
    loc: { start: 1, end: 2 },
    name: 'Dunning',
    kind: 'ArtifactDocument',
    value: 2592000,
    id: 'x',
  });
  assert.equal(
    text,
    '{"kind":"ArtifactDocument","name":"Dunning","value":2592000,"id":"x","loc":{"start":1,"end":2}}',
  );
});

test('pretty JSON follows the same order', () => {
  const text = utils.orderedJsonPretty(
    { loc: { start: 1, end: 2 }, id: 'x', kind: 'ArtifactDocument' },
    2,
  );
  assert.ok(text.startsWith('{\n  "kind": "ArtifactDocument",\n  "id": "x",'), text);
  assert.ok(text.endsWith('\n  "loc": {\n    "start": 1,\n    "end": 2\n  }\n}'), text);
});

test('nodeAtPath walks structural children from the root', () => {
  const ast = {
    kind: 'SourceFile',
    items: [{ kind: 'ArtifactDocument', body: [{ kind: 'PropertyAssignment' }] }],
  };
  assert.equal(utils.nodeAtPath(ast, [0, 0]).kind, 'PropertyAssignment');
  assert.equal(utils.nodeAtPath(ast, [9]), null);
});
