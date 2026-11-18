'use strict';

const vscode = require('vscode');
const { AstExplorer } = require('./ast-view');
const { registerCommands } = require('./commands');
const { ArcholSession } = require('./session');
const { registerLanguageFeatures } = require('./lsp-features');
const { isArcholDocument } = require('./vscode-utils');

const STATUS = {
  stopped: { text: '$(circle-slash) Archol parado', tooltip: 'Servidor LSP parado. Clique para iniciar.' },
  starting: { text: '$(sync~spin) Archol iniciando', tooltip: 'Iniciando o servidor LSP do Archol…' },
  ready: { text: '$(check) Archol pronto', tooltip: 'Servidor LSP do Archol pronto. Clique para reiniciar.' },
  error: { text: '$(error) Archol erro', tooltip: 'Falha no servidor LSP do Archol. Clique para reiniciar.' },
};

let output = null;
let diagnostics = null;
let statusBar = null;
let astExplorer = null;
let session = null;
let extensionVersion = '0.0.0';

function log(message) {
  if (!output) return;
  output.appendLine(`[${new Date().toLocaleTimeString()}] ${message}`);
}

function applyStatus(state) {
  if (!statusBar) return;
  const status = STATUS[state] || STATUS.stopped;
  statusBar.text = status.text;
  statusBar.tooltip = status.tooltip;
}

function activate(context) {
  extensionVersion = context.extension?.packageJSON?.version || '0.0.0';
  output = vscode.window.createOutputChannel('Archol LSP');
  diagnostics = vscode.languages.createDiagnosticCollection('archol');
  statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  statusBar.command = 'archol.restartServer';
  statusBar.show();

  astExplorer = new AstExplorer({
    getAst: (uri) => session.fetchAst(uri),
    log,
    showError: (message) => vscode.window.showErrorMessage(`Archol: ${message}`),
  });

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

  registerCommands(context, { session, astExplorer, output });
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
        log('configuração archol.server.path mudou; reinicie o servidor para aplicar.');
      }
    }),
  );

  applyStatus('stopped');
  log(`Extensão Archol ${extensionVersion} ativada.`);
  for (const document of vscode.workspace.textDocuments) {
    if (isArcholDocument(document)) session.openDocument(document);
  }
}

async function deactivate() {
  if (session) await session.stop();
}

module.exports = { activate, deactivate };
