'use strict';

const vscode = require('vscode');
const {
  applicationDisplayName,
  applicationQuickPickItem,
  applicationUriFromSource,
  isApplicationSource,
} = require('./app-support');
const { applicationSetting } = require('./session-support');
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
  const { session, astExplorer, output, applyApplication } = dependencies;

  const activateApplication = async (uri, application) => {
    try {
      await vscode.workspace
        .getConfiguration('archol')
        .update('application', uri, vscode.ConfigurationTarget.Window);
    } catch (error) {
      vscode.window.showErrorMessage(`Archol: could not save archol.application: ${error.message}`);
      return null;
    }
    const name = applicationDisplayName(application);
    applyApplication?.(uri, name);
    try {
      const applied = await session.setApplication(uri);
      applyApplication?.(uri, applied?.application?.name || name);
      return applied;
    } catch (error) {
      vscode.window.showErrorMessage(
        `Archol: the application was saved, but the server did not confirm it: ${error.message}`,
      );
      return null;
    }
  };

  const selectApplication = async () => {
    let instance;
    try {
      instance = await session.ensureClient();
    } catch {
      // Starting the server already reported the failure.
      return;
    }
    let result;
    try {
      result = await instance.request('archol/applications', {});
    } catch (error) {
      vscode.window.showErrorMessage(`Archol: could not list applications: ${error.message}`);
      return;
    }
    const applications = Array.isArray(result?.applications) ? result.applications : [];
    if (applications.length === 0) {
      vscode.window.showInformationMessage('Archol: no applications found in this workspace.');
      return;
    }
    const configured = applicationSetting();
    const current = typeof result?.current === 'string'
      ? result.current
      : String(result?.current?.uri || configured);
    const items = applications.map((application) => applicationQuickPickItem(application, current));
    const picked = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select the Archol application used for compile and LSP',
      matchOnDescription: true,
      matchOnDetail: true,
    });
    if (!picked) return;
    const application = picked.application || {};
    const uri = String(application.uri || '');
    if (!uri) {
      vscode.window.showErrorMessage('Archol: the selected application has no URI.');
      return;
    }
    const applied = await activateApplication(uri, application);
    if (applied) {
      const lock = applied.lock ? ` Lock: ${applied.lock}.` : '';
      vscode.window.showInformationMessage(
        `Archol: current application is ${applicationDisplayName(application)}.${lock}`,
      );
    }
  };

  // Context-menu entry: use the clicked/active file as the current application.
  const useApplicationFile = async (...args) => {
    const [resource] = normalizeCommandArgs(args);
    const resourceUri = resource && typeof resource === 'object' && typeof resource.toString === 'function'
      ? resource.toString()
      : resource;
    const editor = vscode.window.activeTextEditor;
    const uri = resourceUri
      ? String(resourceUri)
      : editor && isArcholDocument(editor.document)
        ? editor.document.uri.toString()
        : '';
    if (!uri) {
      vscode.window.showInformationMessage('Archol: open an application file first.');
      return;
    }
    let text = '';
    try {
      const document = await vscode.workspace.openTextDocument(vscode.Uri.parse(uri));
      text = document.getText();
    } catch (error) {
      vscode.window.showErrorMessage(`Archol: could not read the file: ${error.message}`);
      return;
    }
    if (!isApplicationSource(text)) {
      vscode.window.showWarningMessage(
        'Archol: this file is not an application (no top-level `application` declaration).',
      );
      return;
    }
    let applications = [];
    try {
      const instance = await session.ensureClient();
      const result = await instance.request('archol/applications', {});
      applications = Array.isArray(result?.applications) ? result.applications : [];
    } catch {
      // The server may still be starting; fall back to the declared URI.
    }
    const fsPath = vscode.Uri.parse(uri).fsPath;
    const declared = applicationUriFromSource(text);
    const application = applications.find(
      (candidate) => candidate.path && vscode.Uri.file(candidate.path).toString() === uri,
    )
      || applications.find((candidate) => candidate.path === fsPath)
      || applications.find((candidate) => candidate.uri === declared)
      || { uri: declared, name: applicationDisplayName({ uri: declared }) };
    if (!application.uri) {
      vscode.window.showErrorMessage('Archol: the application declaration has no URI.');
      return;
    }
    const applied = await activateApplication(application.uri, application);
    if (applied) {
      const lock = applied.lock ? ` Lock: ${applied.lock}.` : '';
      vscode.window.showInformationMessage(
        `Archol: current application is ${applicationDisplayName(application)}.${lock}`,
      );
    }
  };

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

  const updateLock = async () => {
    let instance;
    try {
      instance = await session.ensureClient();
    } catch {
      return;
    }
    try {
      const result = await instance.request('archol/lock', {});
      const count = Number(result?.identifiers) || 0;
      const path = String(result?.path || 'application.lock');
      vscode.window.showInformationMessage(
        `Archol lock updated: ${count} identifier(s) in ${path}.`,
      );
    } catch (error) {
      vscode.window.showErrorMessage(`Archol: could not update the identifier lock: ${error.message}`);
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
    vscode.commands.registerCommand('archol.selectApplication', selectApplication),
    vscode.commands.registerCommand('archol.useApplicationFile', useApplicationFile),
    vscode.commands.registerCommand('archol.updateLock', updateLock),
    vscode.commands.registerCommand('archol.showAst', showAst),
    vscode.commands.registerCommand('archol.showLinks', showLinks),
    vscode.commands.registerCommand('archol.restartServer', restart),
    vscode.commands.registerCommand('archol.showOutput', () => output.show(true)),
  );
}

module.exports = { normalizeCommandArgs, registerCommands };
