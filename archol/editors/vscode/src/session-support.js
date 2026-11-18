'use strict';

const vscode = require('vscode');

function serverSettings() {
  const config = vscode.workspace.getConfiguration('archol');
  const configuredArgs = config.get('server.args', ['lsp']);
  return {
    path: config.get('server.path', 'archol'),
    args: Array.isArray(configuredArgs) ? configuredArgs.map(String) : ['lsp'],
  };
}

function configurationValue(section) {
  const settings = serverSettings();
  if (!section) return {};
  if (section === 'archol.server') return settings;
  if (section === 'archol' || section.startsWith('archol.')) {
    return { server: settings };
  }
  return {};
}

function workspaceFolderList() {
  return (vscode.workspace.workspaceFolders || []).map((folder) => ({
    uri: folder.uri.toString(),
    name: folder.name,
  }));
}

function announceStartError(error, candidates, showOutput) {
  const missing = error.code === 'ENOENT' || /não encontrado|not found/i.test(error.message);
  const message = missing
    ? `Servidor LSP do Archol não encontrado. Configure "archol.server.path" ou compile o servidor (cargo build --release) em archol/. Tentativas: ${candidates.join(', ')}`
    : `Falha ao iniciar o servidor LSP do Archol: ${error.message}`;
  vscode.window.showErrorMessage(message, 'Abrir configurações', 'Mostrar saída').then((action) => {
    if (action === 'Abrir configurações') {
      vscode.commands.executeCommand('workbench.action.openSettings', 'archol.server.path');
    } else if (action === 'Mostrar saída') {
      showOutput();
    }
  });
}

module.exports = { announceStartError, configurationValue, serverSettings, workspaceFolderList };
