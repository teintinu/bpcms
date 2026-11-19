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
3. open an `.archol` file and run **Archol: Select Application** (the status
   bar shows `Archol: no app` until an application is selected);
4. check the status bar (`Archol: <app-name>` once the server is ready).

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
| `archol.application` | `""` | URI of the current Archol application. **Required for compile and LSP**: the compiler and the server work from an application. Run **Archol: Select Application** to choose it; saved with *Window* scope. |
| `archol.server.path` | `archol` | Path to the LSP server executable. |
| `archol.server.args` | `["lsp"]` | Arguments passed to the executable (the `archol` binary uses the `lsp` subcommand). Use `[]` for a dedicated server. |
| `archol.lockDebounceMs` | `10000` | Debounce before refreshing `<app>.lock` after `.archol` changes. |

The selected application URI is sent both in `initialize`
(`initializationOptions.application`) and with `archol/setApplication`. Changing
the setting in **Settings** also notifies the running server; when the server is
stopped, the next initialization picks it up. The value is required for
compilation and LSP features — without it the status bar shows
`Archol: no app` in warning color.

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
  Hover shows the stable identifier id from the application lock.
  `Cmd+Click` (macOS) or `Ctrl+Click` (Windows/Linux) and `F12` use the same
  definition: document field, variable, enum or artifact.
- Hover with an **Open AST** link (`command:archol.showAst?...`) that opens the
  explorer already highlighting the node at the position.
- AST explorer: navigable tree, filter by kind/name, colors by category,
  click to place the cursor, **Go to Code**, **Go to Declaration**,
  **Find Usages** and **Copy JSON**.
  Clicking reuses the already open editor and **selects the whole span**
  (`span.start`..`span.end`), without opening a second tab.
- Identifier navigation from the selected node: the toolbar shows its `id`,
  `ref` and `refKind`; **Go to Declaration** uses `ref` (or `id`) and
  **Find Usages** uses `id`.
- Outside the application closure, the explorer shows a prominent banner
  (*Outside the current application — treated as commented out*) and disables
  the declaration/usages actions.
- Automatic explorer refresh after edits (400 ms debounce).
- **Archol LSP** channel with summarized traffic (method, URI, position,
  diagnostic count). The AST is never logged to the channel.
- Status bar: `Archol: <app-name>` when an application is selected and
  `Archol: no app` (warning color) otherwise, with the LSP state icon
  (`stopped`, `starting`, `ready`, `error`); click to select the application.

## Selecting the application

Three ways, all ending in the same `archol.application` setting:

1. **Archol: Select Application** in the Command Palette (pick from a list);
2. **right-click an application file** in the editor or Explorer and choose
   **Archol: Use as Current Application** (non-application files are rejected);
3. edit `archol.application` directly in Settings.

## Commands and shortcuts

| Command | Palette | Shortcut | Description |
| :--- | :--- | :--- | :--- |
| `archol.selectApplication` | Archol: Select Application | — | Starts the server if needed, lists the applications (`archol/applications`), saves the pick to `archol.application` and calls `archol/setApplication`. |
| `archol.useApplicationFile` | Archol: Use as Current Application | right-click | Right-click an application file (editor or Explorer) and choose this item; the declared URI becomes `archol.application`. |
| `archol.showAst` | Archol: Open AST Explorer | `Ctrl+Alt+A` / `Cmd+Alt+A` | Opens the AST and selects the node at the current position. |
| `archol.showLinks` | Archol: Show Server Links | `Ctrl+Alt+L` / `Cmd+Alt+L` | Requests `archol/links` and opens the response in a JSON document. |
| `archol.updateLock` | Archol: Update Identifier Lock | — | Requests `archol/lock`; creates/prunes `<app>.lock` (CSV `id,path`) with ids for used identifiers; `*` pins entries at the top. |
| `archol.restartServer` | Archol: Restart LSP Server | — | Stops and starts the server; reopens the open files. |
| `archol.showOutput` | Archol: Show Server Output | — | Shows the **Archol LSP** channel. |

The shortcuts apply only to editors with `editorLangId == archol`; the command
palette always works. Clicking the status bar item opens
**Archol: Select Application**; use **Archol: Restart LSP Server** to restart.

## AST explorer

The `archol.astExplorer` panel keeps its context while hidden. The extension
requests `archol/ast` with `{ "uri": "file:///..." }` and renders `ast` as a
single collapsible JSON tree:

- **JSON tree:** every object/array has a twisty; collapsed objects/arrays show
  their contents inline on one line, clipped to the panel width. **Expand all**
  and **Collapse all** are in the toolbar.
- **Key order:** `kind`, `title`, `value`, `message` and `id` always come first
  (in that order); properties ending in `span`/`Span` always come last. The
  same order is used by inline previews and **Copy JSON**.
- **Initial state:** the whole JSON starts collapsed (one line for the root).
  Opening from **Explore AST** expands the selected token and all its parent
  objects, highlights the token and scrolls it into view.
- **Filter** matches JSON keys and scalar values; matching branches expand
  automatically.
- **Click** an AST node object (any JSON object with `kind`/`span`) to select it
  and reveal its span. Clicking a reference never navigates away; it only
  reveals the node's own span.
- **Go to Code** does the same and moves focus to the editor.
- The toolbar shows the selected node identifiers (`id 12 · ref 7 ·
  refKind enumValue`). **Go to Declaration** is enabled when the node has `ref`
  or `id`, requests `archol/declaration { "id": ... }` and reuses the open
  editor to select the returned range. **Find Usages** is enabled when the node
  has `id`, requests `archol/usages`, and opens a QuickPick whose label is
  `relative/path:line` and whose description is the usage `kind`. With no
  results it shows **No usages found**.
- **Copy JSON** copies the selected node or the whole AST.
- The native Cut/Copy/Paste context menu is disabled in the webview.
- When `archol/ast` returns `inApplication: false` (or the AST has
  `commentedOut: true`), a banner reads *Outside the current application —
  treated as commented out* and the declaration/usages buttons are disabled.
- When the open document changes, a new `archol/ast` is requested after 400 ms.

The extension still uses structural index paths to find the node at a cursor
position; clicking a JSON object with `kind`/`span` selects that node in the
JSON tree and reveals its span.

## LSP contract covered

- **Hover**: the server returns Markdown with
  `[Explore AST](command:archol.showAst?<encodeURIComponent(JSON.stringify([uri, line, character]))>)`.
  The command accepts the three arguments (or the single array that VS Code
  delivers), opens the `archol.astExplorer` panel and highlights the node of the
  matching `span`.
- **`initialize`**: includes
  `initializationOptions: { "application": "<uri>" }` from `archol.application`.
- **`archol/applications`**: request `{}` →
  `{ "applications": [{ "uri", "path", "name" }], "current" }`. The QuickPick
  uses `name` as the label and `uri` as the description.
- **`archol/setApplication`**: request `{ "uri": "file:///..." }` →
  `{ "application", "files", "services" }`. Called by the select command and
  whenever `archol.application` changes while the server is running.
- **`archol/ast`**: request `{ "uri": "file:///..." }` →
  `{ "ast": SourceFile, "inApplication": true }`. An AST outside the current
  application is marked with `commentedOut: true`. Spans are converted from
  byte (AST) to UTF-16 position (VS Code) and back.
- **`archol/declaration`**: request `{ "id": <id> }` → `{ "uri", "range" }` or
  `null`. The returned LSP range is selected in the existing editor.
- **`archol/usages`**: request `{ "id": <id> }` →
  `{ "count", "locations": [{ "uri", "range", "kind" }] }`. Labels are built
  from the workspace-relative path and the one-based line.
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
    session-support.js          settings, application, workspaceFolders and spawn error
    app-support.js              pure labels/navigation helpers (app, ranges, usage)
    commands.js                 palette commands, select application and arguments
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
    app-support.test.js         app labels, node identifiers and usage labels
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
- Identifier values (`id`, `ref`, `fieldRef`, `typeRefId`) are treated as opaque
  JSON scalars and forwarded as-is; the extension assumes the server accepts the
  same value it returned in `archol/ast`.
- `archol/declaration` and `archol/usages` return LSP ranges (zero-based
  line/character), unlike AST spans, and are selected directly without byte
  conversion.
- The `current` value of `archol/applications` is accepted as a URI string or as
  an object with `uri`; when absent, `archol.application` is used.
- The server's source of truth is external to this folder: the extension only
  consumes the configured executable.
