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
      if (log) log(`[servidor] ${params.message}`);
      break;
    default:
      vscode.window.showInformationMessage(message);
  }
}

module.exports = { applyDiagnostics, severityOf, showServerMessage, toRange };
