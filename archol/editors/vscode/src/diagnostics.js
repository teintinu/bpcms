'use strict';

const vscode = require('vscode');

const DIAGNOSTIC_SOURCE = 'archol';

function toRange(range) {
  if (!range || !range.start || !range.end) return new vscode.Range(0, 0, 0, 0);
  return new vscode.Range(
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  );
}

function severityOf(value) {
  switch (Number(value)) {
    case 1: return vscode.DiagnosticSeverity.Error;
    case 2: return vscode.DiagnosticSeverity.Warning;
    case 3: return vscode.DiagnosticSeverity.Information;
    case 4: return vscode.DiagnosticSeverity.Hint;
    default: return vscode.DiagnosticSeverity.Error;
  }
}

const { hasOutsideMarker } = require('./diagnostics-support');

const outsideUris = new Set();
let outsideDecoration = null;

function decorationType() {
  if (!outsideDecoration) {
    outsideDecoration = vscode.window.createTextEditorDecorationType({
      fontStyle: 'italic',
      color: new vscode.ThemeColor('editorHint.foreground'),
    });
  }
  return outsideDecoration;
}

function applyOutsideDecorations() {
  const decoration = decorationType();
  for (const editor of vscode.window.visibleTextEditors) {
    const uri = editor.document.uri.toString();
    if (!outsideUris.has(uri)) {
      editor.setDecorations(decoration, []);
      continue;
    }
    const lastLine = editor.document.lineAt(Math.max(0, editor.document.lineCount - 1));
    const range = new vscode.Range(0, 0, lastLine.lineNumber, lastLine.text.length);
    editor.setDecorations(decoration, [{ range, hoverMessage: 'Outside the current application — treated as commented out' }]);
  }
}

function registerOutsideDecorations(context) {
  context.subscriptions.push(
    vscode.window.onDidChangeVisibleTextEditors(() => applyOutsideDecorations()),
    vscode.window.onDidChangeActiveTextEditor(() => applyOutsideDecorations()),
    { dispose: () => { outsideUris.clear(); if (outsideDecoration) outsideDecoration.dispose(); outsideDecoration = null; } },
  );
  applyOutsideDecorations();
}

function applyDiagnostics(collection, params) {
  if (!collection || !params || !params.uri) return;
  const list = Array.isArray(params.diagnostics) ? params.diagnostics : [];
  const mapped = list.map((item) => {
    const diagnostic = new vscode.Diagnostic(
      toRange(item.range),
      String(item.message ?? ''),
      severityOf(item.severity),
    );
    diagnostic.source = item.source || DIAGNOSTIC_SOURCE;
    if (item.code !== undefined && item.code !== null) {
      diagnostic.code = typeof item.code === 'object' ? item.code.value : item.code;
    }
    return diagnostic;
  });
  collection.set(vscode.Uri.parse(params.uri), mapped);
  if (hasOutsideMarker(list)) {
    outsideUris.add(params.uri);
  } else {
    outsideUris.delete(params.uri);
  }
  applyOutsideDecorations();
}

function showServerMessage(params, log) {
  if (!params || !params.message) return;
  const message = `Archol: ${params.message}`;
  switch (Number(params.type)) {
    case 1:
      vscode.window.showErrorMessage(message);
      break;
    case 2:
      vscode.window.showWarningMessage(message);
      break;
    case 4:
      if (log) log(`[server] ${params.message}`);
      break;
    default:
      vscode.window.showInformationMessage(message);
  }
}

module.exports = {
  applyDiagnostics,
  hasOutsideMarker,
  registerOutsideDecorations,
  severityOf,
  showServerMessage,
  toRange,
};
