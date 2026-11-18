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
  const missing = error.code === 'ENOENT' || /not found/i.test(error.message);
  const message = missing
    ? `Archol LSP server not found. Configure "archol.server.path" or build the server (cargo build --release) in archol/. Tried: ${candidates.join(', ')}`
    : `Failed to start the Archol LSP server: ${error.message}`;
  vscode.window.showErrorMessage(message, 'Open Settings', 'Show Output').then((action) => {
    if (action === 'Open Settings') {
      vscode.commands.executeCommand('workbench.action.openSettings', 'archol.server.path');
    } else if (action === 'Show Output') {
      showOutput();
    }
  });
}

module.exports = { announceStartError, configurationValue, serverSettings, workspaceFolderList };
