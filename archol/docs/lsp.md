# LSP

Custom synchronous server over stdio, in `crates/archol-lsp`:

| File | Role |
| :--- | :--- |
| `rpc.rs` | `Content-Length` frames + JSON-RPC |
| `server.rs` | document state, analysis, handlers |
| `features.rs` | AST queries (hover, symbols, context) |
| `util.rs` | URI ↔ path and LSP position (UTF-16) |

## Current application

The server reads `initializationOptions.application` (URI or path) and analyzes
that application's closure. The plugin exposes `archol.application` and the
commands **Archol: Select Application** / **Archol: Update Identifier Lock**.

Files outside the closure are treated as **commented out**: no hover,
completion, definition, references, symbols or folding, a single information
diagnostic (code `W_OUTSIDE_APPLICATION`), `archol/ast` returns
`commentedOut: true`, and the plugin renders the document italic/dimmed. Without an
application the server still starts and reports
`No application selected. Pick one in VS Code (Archol: Select Application).`

## Capabilities

- `textDocumentSync` full (didOpen/didChange/didSave/didClose)
- `hoverProvider` — type, artifact, enum value (with titles), and AST link
- `completionProvider` — context enum values, roles in `roles: [`, artifacts
- `definitionProvider` — goes to the artifact declaration
- `referencesProvider` / `documentHighlightProvider` — occurrences of the identifier
- `documentSymbolProvider` — artifacts and members (hierarchical)
- `workspaceSymbolProvider` — search artifacts by name
- `foldingRangeProvider` — artifacts, blocks, and statements

## Hover

`TaxRegimeCode` shows the enumeration with the titles of each value; `SIMPLES`
shows that it is a value of `TaxRegimeCode`; `Invoice` shows the artifact type
and the service; an application/import alias (`as Identity`) shows the service
URI and jumps to the service file. Every hover also shows the stable lock id
(`id: \`...\``) of the identifier under the cursor — the same id used by the
AST and by the builder. Every hover ends with:

```md
[Explore AST](command:archol.showAst?%5B%22file%3A...%22%2C12%2C4%5D)
```

The plugin opens the AST panel and selects the node at that position.

## Definition

Resolution order (file `crates/archol-lsp/src/resolve.rs`):

1. parameter/`const`/`let` declaration;
2. `icon: X` → `Icon` artifact named `X`;
3. enum value in a typed comparison — `invoice.status == Overdue` resolves
   `status` as a field of `Invoice`, whose type is `InvoiceStatus`, and goes to
   `value Overdue` (not to the `Overdue` icon);
4. `const a: Regime = SIMPLES` → `value SIMPLES`;
5. field access — `invoice.status` → field `status` in the `Invoice` document;
6. same-named variable/artifact/value, respecting the service.

## Completion

In `const a: TaxRegimeCode = `, the extension lists `SIMPLES`,
`LUCRO_PRESUMIDO`, `LUCRO_REAL` with the title of each one. In
`if (regime == ` the list comes from the type of `regime`. In `roles: [` it
lists the roles declared in the workspace. The enum is resolved in the file's
service and accepts `Alias.Type` coming from `imports`; same-named enums from
different services are not mixed.

## Custom requests

| Method | Params | Result |
| :--- | :--- | :--- |
| `archol/ast` | `{ "uri": "file:///..." }` | `{ "ast": <SourceFile>, "inApplication": bool }` |
| `archol/links` | `{}` | `{ "links": <LinkIndex> }` |
| `archol/applications` | `{}` | `{ "applications": [{uri,path,name}], "current": uri|null }` |
| `archol/setApplication` | `{ "uri": "..." }` | `{ "application", "files", "services" }` |
| `archol/status` | `{}` | `{ "application", "files", "services" }` or nulls |
| `archol/declaration` | `{ "id": "..." }` | `{ "uri", "range" }` or `null` |
| `archol/usages` | `{ "id": "..." }` | `{ "count", "locations": [{uri,range,kind}] }` |
| `textDocument/formatting` | `{ textDocument }` | `TextEdit[]` (uppercases pending keywords) |
| `archol/lock` | `{}` | `{ "application", "path", "identifiers" }` (writes the application lock file) |
| `archol/inApplication` | `{ "uri": "file:///..." }` | `{ "inApplication": bool }` |

## Diagnostics

Published on `textDocument/publishDiagnostics` on every open/change/save, with
`source: "archol"` and the same code as the CLI (`E_ENUM_VALUE`,
`E_UNKNOWN_TYPE`, ...). Only the current application closure produces semantic
diagnostics; out-of-closure documents get one information diagnostic.

## Smoke test

```bash
python3 archol/scripts/smoke_lsp.py
```

Starts the real binary with `initializationOptions.application`, runs
`initialize`, hover with an AST link, enum completion, typed references,
`archol/ast`, `archol/links`, and shuts the server down.

## Known limitations

- Application-closure analysis on every change; enough for 172 files in the
  backoffice app, optimizable with per-file caching when necessary.
- `archol/lock` is refreshed automatically on application selection and, after
  `.archol` edits, once every 10 s (debounced by the plugin); the CLI command
  remains for explicit runs.
- Columns in UTF-16 (LSP standard); internal offsets are bytes.
- No `semanticTokens` in this initial version: the TextMate grammar covers
  colors.
- Sync is Full (`change: 1`); incremental changes are not applied.
