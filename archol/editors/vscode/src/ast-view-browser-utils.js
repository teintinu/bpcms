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

  function nodeAtPath(ast, path) {
    let node = ast;
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
    if (node.shape) parts.push(String(node.shape));
    if (node.raw !== undefined && node.kind === 'ShapeLiteral') parts.push(String(node.raw));
    if (node.name) {
      var typeRef = node.typeRef ? ': ' + node.typeRef : '';
      parts.push(String(node.name) + typeRef);
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
  }

  var JSON_FIRST_KEYS = ['kind', 'title', 'value', 'message', 'id'];

  function keyRank(key) {
    var index = JSON_FIRST_KEYS.indexOf(key);
    if (index !== -1) return index;
    if (/span$/i.test(key)) return 1000;
    return 100;
  }

  function orderedKeys(object) {
    return Object.keys(object).sort(function (left, right) {
      return keyRank(left) - keyRank(right);
    });
  }

  function orderedJson(value) {
    if (Array.isArray(value)) {
      return '[' + value.map(orderedJson).join(',') + ']';
    }
    if (value && typeof value === 'object') {
      return '{' + orderedKeys(value).map(function (key) {
        return JSON.stringify(key) + ':' + orderedJson(value[key]);
      }).join(',') + '}';
    }
    return JSON.stringify(value);
  }

  function orderedJsonPretty(value, spaces) {
    var gap = ' '.repeat(spaces || 0);
    function walk(node, level) {
      var pad = gap.repeat(level);
      var childPad = gap.repeat(level + 1);
      if (Array.isArray(node)) {
        if (node.length === 0) return '[]';
        return '[\\n' + node.map(function (item) {
          return childPad + walk(item, level + 1);
        }).join(',\\n') + '\\n' + pad + ']';
      }
      if (node && typeof node === 'object') {
        var keys = orderedKeys(node);
        if (keys.length === 0) return '{}';
        return '{\\n' + keys.map(function (key) {
          return childPad + JSON.stringify(key) + ': ' + walk(node[key], level + 1);
        }).join(',\\n') + '\\n' + pad + '}';
      }
      return JSON.stringify(node);
    }
    return walk(value, 0);
  }
`;

module.exports = { browserUtils };
