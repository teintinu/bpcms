# Archol for VS Code

Official extension for the **Archol** language (`.archol`): syntax highlighting,
snippets, an LSP client written in plain JavaScript and an AST explorer in a
webview.

The extension **has no npm dependencies** and uses only the VS Code API and
native Node.js modules (`child_process`, `crypto`, `events`, `path`). It works
on Windows, macOS and Linux with VS Code **1.80 or later**.

## Installation

### Development (F5)

1. Open the `archol/editors/vscode` folder in VS Code.
2. Press `F5` to open the **Extension Development Host**.
3. Open an `.archol` file from the workspace.

### One-command install (recommended)

```bash
bash archol/scripts/install-vscode.sh
```

The script builds the server (`cargo build --release`) and installs the
extension at `~/.vscode/extensions/archol.archol-vscode-0.1.0` through a symlink
(plugin edits take effect after *Reload Window*). Then:

1. **Developer: Reload Window**;
2. open the repository folder (the binary is found at
   `archol/target/release/archol`);
3. open an `.archol` file and check the status bar (`Archol: ready`).

For VS Code Insiders: `VSCODE_EXTENSIONS_DIR=~/.vscode-insiders/extensions bash archol/scripts/install-vscode.sh`.
Uninstall: `rm ~/.vscode/extensions/archol.archol-vscode-0.1.0`.

### VSIX

The extension does not need a build. If you want to package it:

```bash
npx @vscode/vsce package
```

Then install the `.vsix` in **Extensions → … → Install from VSIX…**.
You can also copy/symlink this folder into `~/.vscode/extensions/`.

### LSP server

The server is a separate executable, built from the Rust code in the
repository:

```bash
cd archol
cargo build --release
```

The binary ends up at `archol/target/release/archol` (or `archol.exe` on
Windows). The extension starts it as `archol lsp`; adjust `archol.server.args`
if needed.

## Configuration

| Setting | Default | Description |
| :--- | :--- | :--- |
| `archol.server.path` | `archol` | Path to the LSP server executable. |
| `archol.server.args` | `["lsp"]` | Arguments passed to the executable (the `archol` binary uses the `lsp` subcommand). Use `[]` for a dedicated server. |

Resolution order:

1. the configured value (absolute, relative to the workspace or a name on `PATH`);
2. `archol/target/release/archol` relative to the workspace root;
3. `archol/target/debug/archol` relative to the workspace root.

If no candidate exists, the extension shows a message with the attempts and
shortcuts to open the settings and the **Archol LSP** channel. After changing
the path, run **Archol: Restart LSP Server**.

## Features

- TextMate syntax highlighting: `//` comments, strings with escapes, numbers
  with suffixes (`5d`, `15m`, `1h`, `10%`), keywords, PascalCase types, calls
  `Name.method(`, constants (`SIMPLES`) and brace folding `{ }`.
- Pair autocomplete, indentation and line comments.
- Snippets: document, process, `base: classification` shape and table view.
- Server diagnostics with severity and code, source `archol`.
- Hover, completion, definition, document symbols and references via LSP.
  `Cmd+Click` (macOS) or `Ctrl+Click` (Windows/Linux) and `F12` use the same
  definition: document field, variable, enum or artifact.
- Hover with an **Open AST** link (`command:archol.showAst?...`) that opens the
  explorer already highlighting the node at the position.
- AST explorer: navigable tree, filter by kind/name, colors by category,
  click to place the cursor, **Go to Code** and **Copy JSON**.
  Clicking reuses the already open editor and **selects the whole span**
  (`span.start`..`span.end`), without opening a second tab.
- Automatic explorer refresh after edits (400 ms debounce).
- **Archol LSP** channel with summarized traffic (method, URI, position,
  diagnostic count). The AST is never logged to the channel.
- Status bar: `stopped`, `starting`, `ready` or `error`; click to restart.

## Commands and shortcuts

| Command | Palette | Shortcut | Description |
| :--- | :--- | :--- | :--- |
| `archol.showAst` | Archol: Open AST Explorer | `Ctrl+Alt+A` / `Cmd+Alt+A` | Opens the AST and selects the node at the current position. |
| `archol.showLinks` | Archol: Show Server Links | `Ctrl+Alt+L` / `Cmd+Alt+L` | Requests `archol/links` and opens the response in a JSON document. |
| `archol.restartServer` | Archol: Restart LSP Server | — | Stops and starts the server; reopens the open files. |
| `archol.showOutput` | Archol: Show Server Output | — | Shows the **Archol LSP** channel. |

The shortcuts apply only to editors with `editorLangId == archol`; the command
palette always works. You can also click the status bar item to restart the
server.

## AST explorer

The `archol.astExplorer` panel keeps its context while hidden. The extension
requests `archol/ast` with `{ "uri": "file:///..." }` and draws `ast` as a tree:

- **Filter** compares the typed text with `kind` and with the summary (`Document Invoice`).
- **Colors** by category (declaration, expression, statement, block) are CSS
  only, using `var(--vscode-*)` to follow the light/dark theme.
- **Click** on a node places the cursor at `span.start` and reveals the span,
  preserving focus in the panel.
- **Go to Code** does the same and moves focus to the editor.
- **Copy JSON** copies the selected node; with no selection, it copies the whole AST.
- When the open document changes, a new `archol/ast` is requested after 400 ms.

Nodes are identified by index paths over structural children (properties that
are nodes and arrays of nodes, in JSON order). The same criterion is used by the
extension (to find the node at a position) and by the webview (to draw the
tree).

## LSP contract covered

- **Hover**: the server returns Markdown with
  `[Explore AST](command:archol.showAst?<encodeURIComponent(JSON.stringify([uri, line, character]))>)`.
  The command accepts the three arguments (or the single array that VS Code
  delivers), opens the `archol.astExplorer` panel and highlights the node of the
  matching `span`.
- **`archol/ast`**: request `{ "uri": "file:///..." }` → `{ "ast": SourceFile }`.
  Spans are converted from byte (AST) to UTF-16 position (VS Code) and back.
- **`archol/links`**: request `{}` → `{ "links": ... }`, displayed as JSON in an
  untitled document.
- **Diagnostics**: `textDocument/publishDiagnostics` with severity 1–4, `code`,
  `source: "archol"` and a range mapped to `vscode.Diagnostic`.
- **Server lifecycle**: `initialize` with the contract capabilities, `initialized`,
  `didOpen`/`didChange`/`didSave`/`didClose` and a response to every request
  sent by the server.

## Architecture

```text
archol/editors/vscode/
  package.json                  manifest, commands, configuration and shortcuts
  language-configuration.json   comments, pairs and indentation
  syntaxes/archol.tmLanguage.json
  snippets/archol.code-snippets
  src/
    extension.js                activation, status bar and VS Code listeners
    session.js                  server lifecycle, document sync and state
    session-support.js          settings, workspaceFolders and spawn error
    commands.js                 palette commands and argument normalization
    diagnostics.js              severity/range mapping and showMessage
    lsp-client.js               JSON-RPC 2.0 over stdio, process and requests
    server-path.js              executable resolution and cross-platform spawn
    lsp-log.js                  traffic summaries without dumping the AST
    protocol.js                 Content-Length framing (encode/decode), pure
    lsp-features.js             hover, completion, definition, symbols and
                                references providers
    vscode-utils.js             Archol document detection
    ast-view.js                 WebviewPanel and bridge with the editor
    ast-view-html.js            inline HTML/CSS shell of the panel (no CDN)
    ast-view-browser.js         webview script (tree, filter and actions)
    ast-view-browser-utils.js   pure helpers for the webview script
    ast-tree.js                 pure AST utilities (offset, path, summary)
  test/
    index.js                    loads the tests for the directory command
    protocol.test.js            framing, fragmentation, UTF-8 and errors
    ast-tree.test.js            tree, spans and position/byte conversion
    server-path.test.js         executable resolution on PATH and fallbacks
```

The client speaks the canonical LSP protocol:

- `Content-Length: N\r\n\r\n` headers and a JSON-RPC 2.0 body in UTF-8;
- `initialize` declares exactly the capabilities for hover, completion,
  `didSave`, hierarchical documentSymbol, definition links and workspaceFolders;
- server requests (`workspace/configuration`,
  `client/registerCapability`, `window/showMessage`, etc.) are always
  answered so the server does not get blocked;
- `textDocument/publishDiagnostics` and `window/showMessage` notifications are
  handled; the rest is safely ignored;
- `didOpen` sends the full text; `didChange` follows the server's
  `textDocumentSync` (incremental with `range`, or full text), and `didSave`
  includes the text;
- the executable starts as `archol lsp` by default; `archol.server.args`
  lets you adjust the arguments (for example, `[]` for a dedicated server);
- spawn errors show instructions, and a process crash does not restart
  automatically: the status bar stays at `error` and there is a **Restart**
  action.

## Development and tests

From the repository:

```bash
for f in archol/editors/vscode/src/*.js; do node --check "$f"; done
node --test archol/editors/vscode/test
```

Inside `archol/editors/vscode`:

```bash
for f in src/*.js; do node --check "$f"; done
node --test test
```

`npm install` is not needed; the tests use only `node:test`,
`node:assert/strict` and native modules.

## Limitations and assumed contracts

- No formatter, semantic tokens, code actions, inlay hints or rename; these
  features are not part of the server's current contract.
- The server may register capabilities dynamically; the extension answers
  `client/registerCapability` with `null`, but does not implement the dynamic
  providers (it only uses the fixed providers above).
- `span.start`/`span.end` are **UTF-8 byte** offsets, as the Archol AST
  documents. The extension converts LSP line/character (UTF-16) → byte and byte
  → JavaScript string offset, handling `\r\n` and surrogate pairs.
- The AST is treated as an acyclic JSON object; circular references are not
  supported.
- The server's source of truth is external to this folder: the extension only
  consumes the configured executable.
