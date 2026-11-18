'use strict';

const vscode = require('vscode');
const { isArcholDocument } = require('./vscode-utils');

function normalizeCommandArgs(args) {
  let values = Array.from(args);
  if (values.length === 1 && Array.isArray(values[0])) values = values[0];
  if (values.length === 1 && typeof values[0] === 'string' && values[0].trim().startsWith('[')) {
    try {
      const parsed = JSON.parse(values[0]);
      if (Array.isArray(parsed)) values = parsed;
    } catch {
      // Keeps the original argument.
    }
  }
  return values;
}

function registerCommands(context, dependencies) {
  const { session, astExplorer, output } = dependencies;

  const showAst = async (...args) => {
    let [uri, line, character] = normalizeCommandArgs(args);
    if (!uri) {
      const editor = vscode.window.activeTextEditor;
      if (!editor || !isArcholDocument(editor.document)) {
        vscode.window.showInformationMessage('Open an .archol file to explore the AST.');
        return;
      }
      uri = editor.document.uri.toString();
      line = editor.selection.active.line;
      character = editor.selection.active.character;
    }
    try {
      await astExplorer.show({
        uri: String(uri),
        line: Number(line) || 0,
        character: Number(character) || 0,
      });
    } catch (error) {
      if (!error.archolShown) {
        vscode.window.showErrorMessage(`Archol: could not open the AST: ${error.message}`);
      }
    }
  };

  const showLinks = async () => {
    let instance;
    try {
      instance = await session.ensureClient();
    } catch {
      return;
    }
    try {
      const result = await instance.request('archol/links', {});
      const content = result === undefined || result === null
        ? 'No links returned by the server.'
        : JSON.stringify(result, null, 2);
      const document = await vscode.workspace.openTextDocument({ language: 'json', content });
      await vscode.window.showTextDocument(document, { preview: false });
    } catch (error) {
      vscode.window.showErrorMessage(`Archol: failed to get links: ${error.message}`);
    }
  };

  const restart = async () => {
    await session.stop();
    try {
      await session.ensureClient();
      vscode.window.showInformationMessage('Archol LSP server restarted.');
    } catch {
      // startServer already showed the error.
    }
  };

  context.subscriptions.push(
    vscode.commands.registerCommand('archol.showAst', showAst),
    vscode.commands.registerCommand('archol.showLinks', showLinks),
    vscode.commands.registerCommand('archol.restartServer', restart),
    vscode.commands.registerCommand('archol.showOutput', () => output.show(true)),
  );
}

module.exports = { normalizeCommandArgs, registerCommands };
