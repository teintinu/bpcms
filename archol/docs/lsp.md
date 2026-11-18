# LSP

Custom synchronous server over stdio, in `crates/archol-lsp`:

| File | Role |
| :--- | :--- |
| `rpc.rs` | `Content-Length` frames + JSON-RPC |
| `server.rs` | document state, analysis, handlers |
| `features.rs` | AST queries (hover, symbols, context) |
| `util.rs` | URI ↔ path and LSP position (UTF-16) |

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
and the service. Every hover ends with:

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
| `archol/ast` | `{ "uri": "file:///..." }` | `{ "ast": <SourceFile> }` |
| `archol/links` | `{}` | `{ "links": <LinkIndex> }` |

## Diagnostics

Published on `textDocument/publishDiagnostics` on every open/change/save, with
`source: "archol"` and the same code as the CLI (`E_ENUM_VALUE`,
`E_UNKNOWN_TYPE`, ...).

## Smoke test

```bash
python3 archol/scripts/smoke_lsp.py
```

Starts the real binary, runs `initialize`, hover with an AST link, enum
completion, `archol/ast`, `archol/links`, and shuts the server down.

## Known limitations

- Whole-workspace analysis on every change; enough for 330 files, optimizable
  with per-file caching when necessary.
- Columns in UTF-16 (LSP standard); internal offsets are bytes.
- No `semanticTokens` in this initial version: the TextMate grammar covers
  colors.
- Sync is Full (`change: 1`); incremental changes are not applied.
