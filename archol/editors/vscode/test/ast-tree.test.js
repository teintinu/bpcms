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
  kind: 'SourceFile',
  path: '/workspace/invoice.archol',
  span: { start: 0, end: text.length },
  items: [
    {
      kind: 'Artifact',
      artifactKind: 'Document',
      name: 'Invoice',
      span: { start: 0, end: text.length },
      body: [
        {
          kind: 'FieldBlock',
          span: { start: fieldsStart, end: totalStart + 'total: Money'.length },
          fields: [
            {
              kind: 'Field',
              name: 'total',
              span: { start: totalStart, end: totalStart + 'total: Money'.length },
              body: [{ kind: 'TypeReference', name: 'Money', span: { start: totalStart + 7, end: totalStart + 12 } }],
            },
          ],
        },
      ],
    },
  ],
};

test('getChildren usa apenas propriedades que são nós', () => {
  assert.deepEqual(getChildren(ast), [ast.items[0]]);
  assert.deepEqual(getChildren(ast.items[0]), [ast.items[0].body[0]]);
  assert.deepEqual(getChildren(ast.items[0].body[0]), [ast.items[0].body[0].fields[0]]);
});

test('nodeAtPath percorre o mesmo caminho usado pelo explorador', () => {
  const field = nodeAtPath(ast, [0, 0, 0]);
  assert.equal(field.kind, 'Field');
  assert.equal(field.name, 'total');
  assert.equal(nodeAtPath(ast, [0, 0, 0, 0]).kind, 'TypeReference');
  assert.equal(nodeAtPath(ast, [9]), null);
  assert.equal(nodeAtPath(ast, null), null);
});

test('findNodeAtOffset escolhe o nó mais profundo que contém o offset', () => {
  assert.equal(findNodeAtOffset(ast, totalStart).node.kind, 'Field');
  assert.deepEqual(findNodePathAtOffset(ast, totalStart), [0, 0, 0]);
  assert.equal(findNodeAtOffset(ast, fieldsStart).node.kind, 'FieldBlock');
  assert.equal(findNodeAtOffset(ast, text.indexOf('document')).node.kind, 'Artifact');
  assert.equal(findNodeAtOffset(ast, text.length - 1).node.kind, 'Artifact');
  assert.equal(findNodeAtOffset(ast, text.length + 50), null);
  assert.equal(findNodePathAtOffset(ast, -1), null);
});

test('findNodeAtOffset aceita offset no fim do span', () => {
  const fieldEnd = totalStart + 'total: Money'.length;
  const found = findNodeAtOffset(ast, fieldEnd);
  assert.ok(found);
  assert.ok(['Field', 'TypeReference', 'FieldBlock', 'Artifact', 'SourceFile'].includes(found.node.kind));
});

test('walk visita a árvore em pré-ordem', () => {
  const visited = [];
  walk(ast, (node, path) => visited.push(`${path.join('.')}:${node.kind}`));
  assert.deepEqual(visited, [
    ':SourceFile',
    '0:Artifact',
    '0.0:FieldBlock',
    '0.0.0:Field',
    '0.0.0.0:TypeReference',
  ]);
});

test('nodeSummary mostra o resumo legível por categoria', () => {
  assert.equal(nodeSummary(ast), 'invoice.archol');
  assert.equal(nodeSummary(ast.items[0]), 'Document Invoice');
  assert.equal(nodeSummary(ast.items[0].body[0]), '');
  assert.equal(nodeSummary(ast.items[0].body[0].fields[0]), 'total');
  assert.equal(nodeSummary({ kind: 'Literal', value: 'SIMPLES' }), 'SIMPLES');
});

test('nodeCategory classifica nós sem depender de CSS', () => {
  assert.equal(nodeCategory('SourceFile'), 'block');
  assert.equal(nodeCategory('FieldBlock'), 'block');
  assert.equal(nodeCategory('Artifact'), 'declaration');
  assert.equal(nodeCategory('FieldDeclaration'), 'declaration');
  assert.equal(nodeCategory('ReturnStatement'), 'statement');
  assert.equal(nodeCategory('IfStatement'), 'statement');
  assert.equal(nodeCategory('BinaryExpression'), 'expression');
  assert.equal(nodeCategory('StringLiteral'), 'expression');
  assert.equal(nodeCategory('Other'), 'other');
});

test('offsetAtText converte linha e character LSP em offset', () => {
  assert.equal(offsetAtText('abc\ndef', 0, 0), 0);
  assert.equal(offsetAtText('abc\ndef', 0, 2), 2);
  assert.equal(offsetAtText('abc\ndef', 1, 1), 5);
  assert.equal(offsetAtText('abc\ndef', 1, 99), 7);
  assert.equal(offsetAtText('abc\ndef', 99, 0), 7);
  assert.equal(offsetAtText('', 0, 0), 0);
});

test('offsetAtText lida com CRLF e com pares substitutos UTF-16', () => {
  assert.equal(offsetAtText('abc\r\ndef\r\n', 1, 2), 7);
  const textWithEmoji = 'a😀b';
  assert.equal(offsetAtText(textWithEmoji, 0, 3), 3);
  assert.equal(offsetAtText(textWithEmoji, 0, 999), 4);
});

test('byteOffsetAtText e textOffsetAtByte convertem UTF-8 e UTF-16', () => {
  const sample = 'a😀bç';
  assert.equal(byteOffsetAtText(sample, 0, 0), 0);
  assert.equal(byteOffsetAtText(sample, 0, 1), 1);
  assert.equal(byteOffsetAtText(sample, 0, 3), 5); // depois do emoji (4 bytes)
  assert.equal(byteOffsetAtText(sample, 0, 4), 6);
  assert.equal(byteOffsetAtText(sample, 0, 5), 8); // ç ocupa 2 bytes
  assert.equal(textOffsetAtByte(sample, 0), 0);
  assert.equal(textOffsetAtByte(sample, 1), 1);
  assert.equal(textOffsetAtByte(sample, 5), 3);
  assert.equal(textOffsetAtByte(sample, 6), 4);
  assert.equal(textOffsetAtByte(sample, 8), 5);
  assert.equal(textOffsetAtByte(sample, 999), sample.length);
  assert.equal(textOffsetAtByte(sample, 2), 1); // no meio do emoji: clampa
});

test('textOffsetsForByteSpan converte início e fim preservando acentos', () => {
  const sample = '// ação\ndocument Invoice {}';
  const startByte = Buffer.byteLength('// ação\n', 'utf8');
  const endByte = Buffer.byteLength('// ação\ndocument Invoice', 'utf8');
  const [start, end] = textOffsetsForByteSpan(sample, startByte, endByte);
  assert.equal(sample.slice(start, end), 'document Invoice');
  assert.equal(start, sample.indexOf('document'));
});

test('textOffsetsForByteSpan normaliza span invertido e fora do texto', () => {
  const sample = 'abcç';
  const [start, end] = textOffsetsForByteSpan(sample, 999, 0);
  assert.equal(start, sample.length);
  assert.equal(end, sample.length);
  const [a, b] = textOffsetsForByteSpan(sample, 3, 5);
  assert.equal(sample.slice(a, b), 'ç');
});
