'use strict';

const { browserScript } = require('./ast-view-browser');

// HTML/CSS/JS inline do explorador de AST. Sem CDN e sem arquivos externos:
// o tema acompanha as variáveis --vscode-* do editor. O script de navegador
// mora em ast-view-browser.js apenas para manter os arquivos pequenos.

function buildAstHtml({ cspSource, nonce }) {
  return `<!DOCTYPE html>
<html lang="pt-BR">
<head>
<meta charset="UTF-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Archol AST</title>
<style>
  :root { color-scheme: light dark; }
  * { box-sizing: border-box; }
  html, body { height: 100%; }
  body {
    margin: 0; display: flex; flex-direction: column;
    background: var(--vscode-editor-background); color: var(--vscode-editor-foreground);
    font-family: var(--vscode-font-family); font-size: var(--vscode-font-size, 13px);
  }
  .toolbar {
    display: flex; flex-wrap: wrap; align-items: center; gap: 6px; padding: 6px 8px;
    border-bottom: 1px solid var(--vscode-panel-border, rgba(128,128,128,.35));
    background: var(--vscode-sideBar-background, transparent);
  }
  .file { font-weight: 600; white-space: nowrap; max-width: 40%; overflow: hidden; text-overflow: ellipsis; }
  #filter {
    flex: 1 1 180px; min-width: 120px; padding: 4px 6px;
    color: var(--vscode-input-foreground); background: var(--vscode-input-background);
    border: 1px solid var(--vscode-input-border, transparent); border-radius: 2px; outline: none;
  }
  #filter:focus { border-color: var(--vscode-focusBorder); }
  button {
    padding: 4px 8px; border: none; border-radius: 2px; cursor: pointer; font-family: inherit;
    color: var(--vscode-button-foreground); background: var(--vscode-button-background);
  }
  button:hover { background: var(--vscode-button-hoverBackground); }
  button:focus { outline: 1px solid var(--vscode-focusBorder); outline-offset: 1px; }
  #status { color: var(--vscode-descriptionForeground); font-size: 11px; min-width: 60px; }
  #tree {
    flex: 1 1 auto; overflow: auto; padding: 6px 8px 24px; white-space: nowrap;
  }
  .node-row {
    display: flex; align-items: center; gap: 5px; padding: 2px 4px;
    border-radius: 2px; cursor: pointer; user-select: none;
  }
  .node-row:hover { background: var(--vscode-list-hoverBackground); }
  .node-row.selected {
    background: var(--vscode-list-activeSelectionBackground);
    color: var(--vscode-list-activeSelectionForeground);
  }
  .twisty { width: 14px; flex: 0 0 14px; text-align: center; color: var(--vscode-descriptionForeground); }
  .twisty.leaf { visibility: hidden; }
  .node-kind {
    font-family: var(--vscode-editor-font-family, monospace); font-size: 11px;
    padding: 0 4px; border-radius: 8px; border: 1px solid currentColor; opacity: .9;
  }
  .node-summary { color: var(--vscode-descriptionForeground); font-size: 12px; margin-left: 2px; }
  .node-row.selected .node-summary { color: inherit; opacity: .85; }
  .children {
    margin-left: 14px; padding-left: 4px;
    border-left: 1px dotted var(--vscode-panel-border, rgba(128,128,128,.35));
  }
  .cat-declaration { color: var(--vscode-charts-blue, #3794ff); }
  .cat-expression { color: var(--vscode-charts-yellow, #d7ba7d); }
  .cat-statement { color: var(--vscode-charts-purple, #c586c0); }
  .cat-block { color: var(--vscode-charts-green, #89d185); }
  .cat-other { color: var(--vscode-descriptionForeground); }
  mark { color: inherit; background: var(--vscode-editor-findMatchHighlightBackground, rgba(234,92,0,.33)); border-radius: 2px; }
  .empty { margin: 16px 4px; color: var(--vscode-descriptionForeground); white-space: normal; }
  .empty.error { color: var(--vscode-errorForeground, #f48771); }
</style>
</head>
<body>
<header class="toolbar">
  <span class="file" id="file-name" title="Arquivo da AST">Archol</span>
  <input id="filter" type="search" placeholder="Filtrar por kind ou nome" aria-label="Filtrar nós da AST">
  <button id="reveal" type="button" title="Posiciona o cursor no início do nó selecionado">Ir para código</button>
  <button id="copy" type="button" title="Copia o JSON do nó selecionado (ou da AST inteira)">Copiar JSON</button>
  <span id="status" role="status"></span>
</header>
<main id="tree" role="tree" aria-label="Árvore de AST do Archol"></main>
<script nonce="${nonce}">
${browserScript}
</script>
</body>
</html>`;
}

module.exports = { buildAstHtml };
