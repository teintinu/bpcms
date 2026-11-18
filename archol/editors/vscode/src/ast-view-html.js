'use strict';

const { browserScript } = require('./ast-view-browser');

// Inline HTML/CSS/JS for the AST explorer. No CDN and no external files:
// the theme follows the editor's --vscode-* variables. The browser script
// lives in ast-view-browser.js just to keep the files small.

function buildAstHtml({ cspSource, nonce }) {
  return `<!DOCTYPE html>
<html lang="en">
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
  button:disabled {
    opacity: .45; cursor: default;
    background: var(--vscode-button-secondaryBackground, var(--vscode-button-background));
  }
  .secondary {
    color: var(--vscode-button-secondaryForeground, var(--vscode-button-foreground));
    background: var(--vscode-button-secondaryBackground, var(--vscode-button-background));
  }
  .secondary:hover { background: var(--vscode-button-secondaryHoverBackground, var(--vscode-button-hoverBackground)); }
  #status { color: var(--vscode-descriptionForeground); font-size: 11px; min-width: 60px; }
  #selection {
    flex: 1 1 100%; order: 10; color: var(--vscode-descriptionForeground);
    font-family: var(--vscode-editor-font-family, monospace); font-size: 11px;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  #selection:empty { display: none; }
  #banner {
    padding: 8px 10px; font-weight: 600;
    color: var(--vscode-inputValidation-warningForeground, var(--vscode-editor-foreground));
    background: var(--vscode-inputValidation-warningBackground, rgba(255, 204, 0, .2));
    border-bottom: 1px solid var(--vscode-inputValidation-warningBorder, var(--vscode-panel-border));
  }
  #banner[hidden] { display: none; }
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
  <span class="file" id="file-name" title="AST file">Archol</span>
  <input id="filter" type="search" placeholder="Filter by kind or name" aria-label="Filter AST nodes">
  <button id="reveal" type="button" title="Places the cursor at the start of the selected node">Go to Code</button>
  <button id="declaration" type="button" class="secondary" title="Opens the declaration of the selected node reference">Go to Declaration</button>
  <button id="usages" type="button" class="secondary" title="Lists the usages of the selected declaration">Find Usages</button>
  <button id="copy" type="button" title="Copies the JSON of the selected node (or the whole AST)">Copy JSON</button>
  <span id="status" role="status"></span>
  <span id="selection" title="Selected node identifiers"></span>
</header>
<div id="banner" role="alert" hidden>Outside the current application — treated as commented out</div>
<main id="tree" role="tree" aria-label="Archol AST tree"></main>
<script nonce="${nonce}">
${browserScript}
</script>
</body>
</html>`;
}

module.exports = { buildAstHtml };
