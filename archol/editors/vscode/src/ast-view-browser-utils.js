'use strict';

// Pure functions for the webview script (no reference to tree state).
const browserUtils = `  function isNode(value) {
    return Boolean(value) && typeof value === 'object' && typeof value.kind === 'string';
  }

  function appendHighlighted(parent, text, term) {
    if (!term) {
      parent.appendChild(document.createTextNode(text));
      return;
    }
    const lower = text.toLowerCase();
    let position = 0;
    while (position < text.length) {
      const found = lower.indexOf(term, position);
      if (found === -1) {
        parent.appendChild(document.createTextNode(text.slice(position)));
        break;
      }
      if (found > position) parent.appendChild(document.createTextNode(text.slice(position, found)));
      const mark = document.createElement('mark');
      mark.textContent = text.slice(found, found + term.length);
      parent.appendChild(mark);
      position = found + term.length;
    }
  }
  function childrenOf(node) {
    const children = [];
    if (!node) return children;
    Object.keys(node).forEach(function (key) {
      if (key === 'span' || key === 'kind') return;
      const value = node[key];
      if (Array.isArray(value)) {
        value.forEach(function (item) { if (isNode(item)) children.push(item); });
      } else if (isNode(value)) {
        children.push(value);
      }
    });
    return children;
  }

  function nodeAtPath(path) {
    let node = state.ast;
    if (!node || !Array.isArray(path)) return null;
    for (let index = 0; index < path.length; index += 1) {
      node = childrenOf(node)[path[index]];
      if (!node) return null;
    }
    return node;
  }

  function pathKey(path) { return path.join('/'); }

  function summaryOf(node) {
    if (!node) return '';
    const parts = [];
    if (node.artifactKind) parts.push(String(node.artifactKind));
    if (node.name) {
      parts.push(String(node.name));
    } else if (node.kind === 'SourceFile' && node.path) {
      const segments = String(node.path).split(/[\\\\/]/);
      parts.push(segments[segments.length - 1]);
    }
    if (parts.length === 0 && node.value !== undefined && node.value !== null && typeof node.value !== 'object') {
      parts.push(String(node.value));
    }
    return parts.join(' ');
  }

  function categoryOf(kind) {
    if (/Block$|Body$|^SourceFile$/.test(kind)) return 'block';
    if (/Declaration$|^Artifact$|^Field$|^Parameter$|^Transition$|^Value$|^Role$|^Action$|^PropertyAssignment$|^ShorthandProperty$/.test(kind)) return 'declaration';
    if (/Statement$|^Return|^If|^Send|^Try|^Catch|^Let|^Const/.test(kind)) return 'statement';
    if (/Expression$|Literal$|^Call|^Reference|^Binary|^Unary|^Identifier$|^MemberAccess$|^List$|^Object$|^NamedArgument$/.test(kind)) return 'expression';
    return 'other';
  }

  function countNodes(node) {
    let total = 1;
    childrenOf(node).forEach(function (child) { total += countNodes(child); });
    return total;
  }`;

module.exports = { browserUtils };
