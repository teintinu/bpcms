'use strict';

const vscode = require('vscode');
const { AstExplorer } = require('./ast-view');
const { registerCommands } = require('./commands');
const { registerOutsideDecorations } = require('./diagnostics');
const { ArcholSession } = require('./session');
const { registerLanguageFeatures } = require('./lsp-features');
const { applicationNameFromUri } = require('./app-support');
const { applicationSetting } = require('./session-support');
const { isArcholDocument } = require('./vscode-utils');

const STATUS = {
  stopped: { icon: '$(circle-slash)', label: 'LSP server stopped' },
  starting: { icon: '$(sync~spin)', label: 'Starting the LSP server…' },
  ready: { icon: '$(check)', label: 'LSP server ready' },
  error: { icon: '$(error)', label: 'LSP server failed' },
};

let output = null;
let diagnostics = null;
let statusBar = null;
let astExplorer = null;
let session = null;
let extensionVersion = '0.0.0';
let serverState = 'stopped';
let applicationUri = '';
let applicationName = '';

function log(message) {
  if (!output) return;
  output.appendLine(`[${new Date().toLocaleTimeString()}] ${message}`);
}

function renderStatusBar() {
  if (!statusBar) return;
  const status = STATUS[serverState] || STATUS.stopped;
  const label = applicationUri
    ? (applicationName || applicationNameFromUri(applicationUri) || 'no app')
    : 'no app';
  statusBar.text = `${status.icon} Archol: ${label}`;
  const tooltip = new vscode.MarkdownString();
  tooltip.appendMarkdown(`**Archol LSP:** ${status.label}\n\n`);
  if (applicationUri) {
    tooltip.appendMarkdown(`Application: **${label}**\n\n\`${applicationUri}\`\n\n`);
  } else {
    tooltip.appendMarkdown(
      'No application selected. An application is required to compile and for LSP features.\n\n',
    );
  }
  tooltip.appendMarkdown('Click to select the current application.');
  statusBar.tooltip = tooltip;
  statusBar.command = 'archol.selectApplication';
  if (!applicationUri) {
    statusBar.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
  } else if (serverState === 'error') {
    statusBar.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
  } else {
    statusBar.backgroundColor = undefined;
  }
}

function applyStatus(state) {
  serverState = STATUS[state] ? state : 'stopped';
  renderStatusBar();
}

// Updates the app part of the status bar; `name` comes from the server when
// available and otherwise is derived from the application URI.
function applyApplication(uri, name) {
  applicationUri = typeof uri === 'string' ? uri.trim() : '';
  applicationName = applicationUri && typeof name === 'string' ? name.trim() : '';
  renderStatusBar();
}

function activate(context) {
  extensionVersion = context.extension?.packageJSON?.version || '0.0.0';
  output = vscode.window.createOutputChannel('Archol LSP');
  diagnostics = vscode.languages.createDiagnosticCollection('archol');
  statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  statusBar.command = 'archol.selectApplication';
  statusBar.show();
  applyApplication(applicationSetting());

  astExplorer = new AstExplorer({
    getAst: (uri) => session.fetchAst(uri),
    getDeclaration: (id) => session.fetchDeclaration(id),
    getUsages: (id) => session.fetchUsages(id),
    log,
    showError: (message) => vscode.window.showErrorMessage(`Archol: ${message}`),
    showInfo: (message) => vscode.window.showInformationMessage(message),
  });

  registerOutsideDecorations(context);

  session = new ArcholSession({
    diagnostics,
    log,
    extensionVersion,
    showOutput: () => output.show(true),
    setState: applyStatus,
    getAstExplorer: () => astExplorer,
  });

  context.subscriptions.push(output, diagnostics, statusBar, astExplorer, {
    dispose: () => {
      if (session) session.stop();
    },
  });

  registerCommands(context, { session, astExplorer, output, applyApplication });
  registerLanguageFeatures(context, {
    getClient: () => session.activeClient(),
    log,
  });

  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((document) => session.openDocument(document)),
    vscode.workspace.onDidChangeTextDocument((event) => session.changeDocument(event)),
    vscode.workspace.onDidSaveTextDocument((document) => session.saveDocument(document)),
    vscode.workspace.onDidCloseTextDocument((document) => session.closeDocument(document)),
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor && isArcholDocument(editor.document)) session.openDocument(editor.document);
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('archol.server.path')) {
        log('archol.server.path setting changed; restart the server to apply.');
      }
      if (event.affectsConfiguration('archol.application')) {
        const uri = applicationSetting();
        applyApplication(uri);
        log(`archol.application changed: ${uri || '(empty)'}`);
        session
          .applyConfiguredApplication()
          .then((result) => {
            if (result) applyApplication(uri, result?.application?.name);
          })
          .catch((error) => {
            log(`failed to apply archol.application: ${error.message}`);
          });
      }
    }),
  );

  applyStatus('stopped');
  log(`Archol extension ${extensionVersion} activated.`);
  for (const document of vscode.workspace.textDocuments) {
    if (isArcholDocument(document)) session.openDocument(document);
  }
}

async function deactivate() {
  if (session) await session.stop();
}

module.exports = { activate, deactivate };
