'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const {
  byteOffsetAtText,
  findNodeAtOffset,
  findNodePathAtOffset,
  getChildren,
  nodeAtPath,
  nodeCategory,
  nodeSummary,
  offsetAtText,
  textOffsetAtByte,
  textOffsetsForByteSpan,
  walk,
} = require('../src/ast-tree');

const text = 'document Invoice {\n  fields {\n    total: Money\n  }\n}\n';
const fieldsStart = text.indexOf('fields');
const totalStart = text.indexOf('total');
const ast = {
  kind: 'ArcholApplication',
  path: '/workspace/invoice.archol',
  items: [
    {
      kind: 'ArtifactDocument',
      name: 'Invoice',
      loc: { start: 0, end: text.length },
      body: [
        {
          kind: 'FieldBlock',
          loc: { start: fieldsStart, end: totalStart + 'total: Money'.length },
          fields: [
            {
              kind: 'Field',
              name: 'total',
              loc: { start: totalStart, end: totalStart + 'total: Money'.length },
              body: [{ kind: 'TypeReference', name: 'Money', loc: { start: totalStart + 7, end: totalStart + 12 } }],
            },
          ],
        },
      ],
    },
  ],
};

test('getChildren uses only properties that are nodes', () => {
  assert.deepEqual(getChildren(ast), [ast.items[0]]);
  assert.deepEqual(getChildren(ast.items[0]), [ast.items[0].body[0]]);
  assert.deepEqual(getChildren(ast.items[0].body[0]), [ast.items[0].body[0].fields[0]]);
});

test('nodeAtPath follows the same path used by the explorer', () => {
  const field = nodeAtPath(ast, [0, 0, 0]);
  assert.equal(field.kind, 'Field');
  assert.equal(field.name, 'total');
  assert.equal(nodeAtPath(ast, [0, 0, 0, 0]).kind, 'TypeReference');
  assert.equal(nodeAtPath(ast, [9]), null);
  assert.equal(nodeAtPath(ast, null), null);
});

test('findNodeAtOffset picks the deepest node containing the offset', () => {
  assert.equal(findNodeAtOffset(ast, totalStart).node.kind, 'Field');
  assert.deepEqual(findNodePathAtOffset(ast, totalStart), ['items', 0, 'body', 0, 'fields', 0]);
  assert.equal(findNodeAtOffset(ast, fieldsStart).node.kind, 'FieldBlock');
  assert.equal(findNodeAtOffset(ast, text.indexOf('document')).node.kind, 'ArtifactDocument');
  assert.equal(findNodeAtOffset(ast, text.length - 1).node.kind, 'ArtifactDocument');
  assert.equal(findNodeAtOffset(ast, text.length + 50), null);
  assert.equal(findNodePathAtOffset(ast, -1), null);
});

test('findNodeAtOffset accepts an offset at the end of the loc', () => {
  const fieldEnd = totalStart + 'total: Money'.length;
  const found = findNodeAtOffset(ast, fieldEnd);
  assert.ok(found);
  assert.ok(['Field', 'TypeReference', 'FieldBlock', 'ArtifactDocument', 'ArcholApplication'].includes(found.node.kind));
});

test('walk visits the tree in pre-order', () => {
  const visited = [];
  walk(ast, (node, path) => visited.push(`${path.join('.')}:${node.kind}`));
  assert.deepEqual(visited, [
    ':ArcholApplication',
    '0:ArtifactDocument',
    '0.0:FieldBlock',
    '0.0.0:Field',
    '0.0.0.0:TypeReference',
  ]);
});

test('nodeSummary shows the readable summary per category', () => {
  assert.equal(nodeSummary(ast), 'invoice.archol');
  assert.equal(nodeSummary(ast.items[0]), 'Document Invoice');
  assert.equal(nodeSummary(ast.items[0].body[0]), '');
  assert.equal(nodeSummary(ast.items[0].body[0].fields[0]), 'total');
  assert.equal(nodeSummary({ kind: 'Literal', value: 'SIMPLES' }), 'SIMPLES');
  assert.equal(
    nodeSummary({ kind: 'PropertyAssignment', name: 'goal', typeRef: 'I18N' }),
    'goal: I18N',
  );
  assert.equal(
    nodeSummary({ kind: 'ShapeLiteral', shape: 'std.Duration', raw: '30d', value: 2592000 }),
    'std.Duration 30d',
  );
});

test('nodeCategory classifies nodes without relying on CSS', () => {
  assert.equal(nodeCategory('ArcholApplication'), 'block');
  assert.equal(nodeCategory('FieldBlock'), 'block');
  assert.equal(nodeCategory('ArtifactDocument'), 'declaration');
  assert.equal(nodeCategory('FieldDeclaration'), 'declaration');
  assert.equal(nodeCategory('ReturnStatement'), 'statement');
  assert.equal(nodeCategory('IfStatement'), 'statement');
  assert.equal(nodeCategory('BinaryExpression'), 'expression');
  assert.equal(nodeCategory('StringLiteral'), 'expression');
  assert.equal(nodeCategory('Other'), 'other');
});

test('offsetAtText converts an LSP line and character into an offset', () => {
  assert.equal(offsetAtText('abc\ndef', 0, 0), 0);
  assert.equal(offsetAtText('abc\ndef', 0, 2), 2);
  assert.equal(offsetAtText('abc\ndef', 1, 1), 5);
  assert.equal(offsetAtText('abc\ndef', 1, 99), 7);
  assert.equal(offsetAtText('abc\ndef', 99, 0), 7);
  assert.equal(offsetAtText('', 0, 0), 0);
});

test('offsetAtText handles CRLF and UTF-16 surrogate pairs', () => {
  assert.equal(offsetAtText('abc\r\ndef\r\n', 1, 2), 7);
  const textWithEmoji = 'a😀b';
  assert.equal(offsetAtText(textWithEmoji, 0, 3), 3);
  assert.equal(offsetAtText(textWithEmoji, 0, 999), 4);
});

test('byteOffsetAtText and textOffsetAtByte convert between UTF-8 and UTF-16', () => {
  const sample = 'a😀bç';
  assert.equal(byteOffsetAtText(sample, 0, 0), 0);
  assert.equal(byteOffsetAtText(sample, 0, 1), 1);
  assert.equal(byteOffsetAtText(sample, 0, 3), 5); // after the emoji (4 bytes)
  assert.equal(byteOffsetAtText(sample, 0, 4), 6);
  assert.equal(byteOffsetAtText(sample, 0, 5), 8); // ç takes 2 bytes
  assert.equal(textOffsetAtByte(sample, 0), 0);
  assert.equal(textOffsetAtByte(sample, 1), 1);
  assert.equal(textOffsetAtByte(sample, 5), 3);
  assert.equal(textOffsetAtByte(sample, 6), 4);
  assert.equal(textOffsetAtByte(sample, 8), 5);
  assert.equal(textOffsetAtByte(sample, 999), sample.length);
  assert.equal(textOffsetAtByte(sample, 2), 1); // inside the emoji: clamps
});

test('textOffsetsForByteSpan converts start and end while preserving accents', () => {
  const sample = '// ação\ndocument Invoice {}';
  const startByte = Buffer.byteLength('// ação\n', 'utf8');
  const endByte = Buffer.byteLength('// ação\ndocument Invoice', 'utf8');
  const [start, end] = textOffsetsForByteSpan(sample, startByte, endByte);
  assert.equal(sample.slice(start, end), 'document Invoice');
  assert.equal(start, sample.indexOf('document'));
});

test('textOffsetsForByteSpan normalizes an inverted range and out-of-range values', () => {
  const sample = 'abcç';
  const [start, end] = textOffsetsForByteSpan(sample, 999, 0);
  assert.equal(start, sample.length);
  assert.equal(end, sample.length);
  const [a, b] = textOffsetsForByteSpan(sample, 3, 5);
  assert.equal(sample.slice(a, b), 'ç');
});

test('findNodeAtOffset filters by the file id when provided', () => {
  const withFiles = JSON.parse(JSON.stringify(ast));
  withFiles.items[0].loc.file = 'f:a';
  withFiles.items[0].body[0].loc.file = 'f:a';
  withFiles.items[0].body[0].fields[0].loc.file = 'f:a';
  withFiles.items[0].body[0].fields[0].body[0].loc.file = 'f:a';
  assert.equal(findNodeAtOffset(withFiles, totalStart, 'f:a').node.kind, 'Field');
  assert.equal(findNodeAtOffset(withFiles, totalStart, 'f:b'), null);
});

test('findNodeAtOffset walks artifact maps and returns raw JSON paths', () => {
  const app = {
    kind: 'ArcholApplication',
    artifact: {
      i: { kind: 'ProcessDef', loc: { file: 'f:b', start: 0, end: 10 } },
    },
  };
  assert.deepEqual(findNodePathAtOffset(app, 5, 'f:b'), ['artifact', 'i']);
  assert.equal(findNodeAtOffset(app, 5, 'f:a'), null);
});

test('findNodeAtOffset prefers the outer node when spans are equal', () => {
  const nested = {
    kind: 'Root',
    items: [
      {
        kind: 'Outer',
        loc: { start: 0, end: 10 },
        child: { kind: 'Inner', loc: { start: 0, end: 10 } },
      },
    ],
  };
  assert.equal(findNodeAtOffset(nested, 5).node.kind, 'Outer');
});
