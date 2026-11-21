'use strict';

// Pure utilities over the AST returned by `archol/ast`.
// The child order is the same one used by the explorer (src/ast-view-html.js):
// properties that are nodes, then arrays of nodes, in JSON object order.

const META_KEYS = new Set(['kind', 'loc']);

function isAstNode(value) {
  return Boolean(value) && typeof value === 'object' && typeof value.kind === 'string';
}

function getChildren(node) {
  const children = [];
  if (!isAstNode(node)) return children;
  for (const key of Object.keys(node)) {
    if (META_KEYS.has(key)) continue;
    const value = node[key];
    if (Array.isArray(value)) {
      for (const item of value) {
        if (isAstNode(item)) children.push(item);
      }
    } else if (isAstNode(value)) {
      children.push(value);
    }
  }
  return children;
}


// The server returns comments in `SourceFile.pending`. The explorer shows them
// as children of the deepest node that contains them, in source order, so the
// tree follows the file instead of listing every pending at the end.
function mergePending(ast) {
  if (!isAstNode(ast) || !Array.isArray(ast.pending)) return ast;
  const pending = ast.pending
    .filter(isAstNode)
    .filter((node) => node.kind === 'Pending')
    .sort((left, right) => spanStart(left) - spanStart(right));
  delete ast.pending;
  for (const item of pending) {
    const container = pendingContainer(ast, item.loc, null);
    if (!container) continue;
    const target = chooseArray(container, item.loc);
    if (!target) continue;
    target.array.splice(insertIndex(target.array, item.loc), 0, item);
  }
  return ast;
}

function spanStart(node) {
  return node && node.loc && Number.isFinite(node.loc.start) ? node.loc.start : 0;
}

function contains(span, point) {
  return Boolean(span)
    && Number.isFinite(span.start)
    && Number.isFinite(span.end)
    && Number.isFinite(point)
    && point >= span.start
    && point <= span.end;
}

// Deepest node with a child array that contains the pending comment.
function pendingContainer(node, span, fallback) {
  const children = getChildren(node);
  for (const child of children) {
    if (!isAstNode(child) || child.kind === 'Pending') continue;
    if (!child.loc || !contains(child.loc, span.start)) continue;
    const deeper = pendingContainer(child, span, fallback);
    if (deeper && arrayKeys(deeper).length > 0) return deeper;
  }
  if (arrayKeys(node).length > 0) return node;
  return fallback;
}

function arrayKeys(node) {
  return Object.keys(node).filter((key) => {
    if (key === 'pending' || key === 'loc' || key === 'keywordLoc') return false;
    const value = node[key];
    return Array.isArray(value) && value.some((item) => isAstNode(item));
  });
}

function chooseArray(node, span) {
  const keys = arrayKeys(node);
  if (keys.length === 0) return null;
  let best = null;
  for (const key of keys) {
    const array = node[key];
    let distance = Infinity;
    for (const item of array) {
      if (!isAstNode(item) || !item.loc) continue;
      if (contains(item.loc, span.start)) {
        distance = 0;
        break;
      }
      distance = Math.min(distance, Math.abs(item.loc.start - span.start));
    }
    if (!best || distance < best.distance) best = { array, distance };
  }
  return best;
}

function insertIndex(array, span) {
  for (let index = 0; index < array.length; index += 1) {
    const item = array[index];
    if (isAstNode(item) && item.loc && item.loc.start > span.start) return index;
  }
  return array.length;
}

function nodeAtPath(root, path) {
  let node = root;
  if (!Array.isArray(path)) return null;
  for (const index of path) {
    if (!Number.isInteger(index) || index < 0) return null;
    node = getChildren(node)[index];
    if (!node) return null;
  }
  return node;
}

function walk(node, visit, path = []) {
  if (!isAstNode(node)) return;
  visit(node, path);
  const children = getChildren(node);
  for (let index = 0; index < children.length; index += 1) {
    walk(children[index], visit, path.concat(index));
  }
}

function findNodeAtOffset(root, offset) {
  if (!isAstNode(root) || !Number.isFinite(offset)) return null;
  let best = null;
  walk(root, (node, path) => {
    const span = node.loc;
    if (!span || !Number.isFinite(span.start) || !Number.isFinite(span.end)) return;
    if (offset < span.start || offset > span.end) return;
    const length = span.end - span.start;
    if (
      best === null
      || path.length > best.path.length
      || (path.length === best.path.length && length < best.length)
    ) {
      best = { node, path, length };
    }
  });
  return best;
}

function findNodePathAtOffset(root, offset) {
  const found = findNodeAtOffset(root, offset);
  return found ? found.path : null;
}

function artifactKindOf(node) {
  if (!isAstNode(node)) return '';
  const kind = node.kind;
  if (typeof kind === 'string' && kind.startsWith('Artifact') && kind !== 'Artifact') {
    return kind.slice('Artifact'.length);
  }
  return '';
}

function nodeSummary(node) {
  if (!isAstNode(node)) return '';
  const parts = [];
  const artifactKind = artifactKindOf(node);
  if (artifactKind !== '') parts.push(artifactKind);
  if (node.shape) parts.push(String(node.shape));
  if (node.raw !== undefined && node.kind === 'ShapeLiteral') parts.push(String(node.raw));
  if (node.name) {
    const typeRef = node.typeRef ? `: ${node.typeRef}` : '';
    parts.push(`${node.name}${typeRef}`);
  } else if (node.kind === 'SourceFile' && node.path) {
    const segments = String(node.path).split(/[\\/]/);
    parts.push(segments[segments.length - 1]);
  }
  if (parts.length === 0 && node.value !== undefined && node.value !== null && typeof node.value !== 'object') {
    parts.push(String(node.value));
  }
  return parts.join(' ');
}

function nodeCategory(kind) {
  if (/Block$|Body$|^SourceFile$|^Hierarchy$|^HierarchyChain$/.test(kind)) return 'block';
  if (/Declaration$|^Artifact|^Field$|^Parameter$|^Transition$|^Value$|^Role$|^Action$|^PropertyAssignment$|^ShorthandProperty$/.test(kind)) {
    return 'declaration';
  }
  if (/Statement$|^Return|^If|^Send|^Try|^Catch|^Let|^Const/.test(kind)) return 'statement';
  if (/Expression$|Literal$|^Call|^Reference|^Binary|^Unary|^Identifier$|^MemberAccess$|^List$|^Object$|^NamedArgument$/.test(kind)) return 'expression';
  return 'other';
}

// Converts an LSP position (zero-based line/character, UTF-16) into a JavaScript
// string offset. `\r` from CRLF does not count as line content.
function offsetAtText(text, line, character) {
  const source = typeof text === 'string' ? text : '';
  const targetLine = Math.max(0, Math.trunc(Number(line) || 0));
  const targetCharacter = Math.max(0, Math.trunc(Number(character) || 0));
  const lines = source.split('\n');
  let offset = 0;
  for (let index = 0; index < targetLine && index < lines.length; index += 1) {
    offset += lines[index].length + 1;
  }
  if (targetLine >= lines.length) return source.length;
  let lineText = lines[targetLine];
  if (lineText.endsWith('\r')) lineText = lineText.slice(0, -1);
  return offset + Math.min(targetCharacter, lineText.length);
}

// The Archol AST uses byte spans in UTF-8; VS Code uses JavaScript string
// offsets (UTF-16). The two functions below bridge them.
function byteOffsetAtText(text, line, character) {
  const source = typeof text === 'string' ? text : '';
  return Buffer.byteLength(source.slice(0, offsetAtText(source, line, character)), 'utf8');
}

function textOffsetAtByte(text, byteOffset) {
  const source = typeof text === 'string' ? text : '';
  const target = Math.max(0, Math.trunc(Number(byteOffset) || 0));
  let bytes = 0;
  for (let index = 0; index < source.length;) {
    const codePoint = source.codePointAt(index);
    const character = String.fromCodePoint(codePoint);
    const size = Buffer.byteLength(character, 'utf8');
    if (bytes + size > target) return index;
    bytes += size;
    index += character.length;
    if (bytes === target) return index;
  }
  return source.length;
}

// Converts an AST byte span into UTF-16 offsets used by VS Code.
function textOffsetsForByteSpan(text, startByte, endByte) {
  const start = textOffsetAtByte(text, startByte);
  const end = Math.max(start, textOffsetAtByte(text, endByte));
  return [start, end];
}

module.exports = {
  mergePending,
  byteOffsetAtText,
  textOffsetsForByteSpan,
  findNodeAtOffset,
  findNodePathAtOffset,
  getChildren,
  isAstNode,
  nodeAtPath,
  nodeCategory,
  nodeSummary,
  offsetAtText,
  textOffsetAtByte,
  walk,
};
