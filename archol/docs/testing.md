# Tests

```bash
cd archol
cargo test                 # everything
cargo test -p archol-core  # lexer + parser
cargo test -p archol-analysis
node --test editors/vscode/test
```

## What is covered

| Test | Guarantees |
| :--- | :--- |
| `archol-core/src/lexer.rs` (inline tests) | strings, comments, numbers with a suffix, 3-character operators, escape at EOF |
| `archol-core/tests/parser.rs` | artifacts, enum, const/if, lists, tolerance, unclosed block error, `E_PARSE_DEPTH` |
| `archol-analysis/tests/analysis.rs` | `E_ENUM_VALUE` in const and comparison (including `Alias.Type`), same-named enum per service, `E_UNKNOWN_ROLE`, `E_DUP_ARTIFACT`, role→artifact link |
| `archol-core/tests/corpus.rs` | the 330 real examples parse without error |
| `archol-cli/tests/cli.rs` | exit codes, `E_FILE_READ` for invalid UTF-8, `tokens` with trailing `\`, deep nesting |
| `archol-analysis/tests/annotate.rs` | typed references (`Reference`/`ref`), field refs, lock ids and pruning |
| `archol-analysis/src/ids.rs` (inline tests) | sequential used-only ids, usage-first allocation, base36, CSV roundtrip, pinned `*`, duplicate rejection |
| `archol-analysis/src/application.rs` (inline tests) | application discovery, transitive imports, closure file set |
| `archol-lsp/src/resolve.rs` + `server/tests.rs` | field/enum definitions, app selection, declaration/usages by id |
| `archol-cli/tests/cli.rs` | `--app` required, `applications`, `lock` create/stable/prune, annotated `ast`, `format --check`, pending diagnostics |
| `archol-cli/tests/ast_snapshots.rs` | every example application against `<app>.expected-ast.yml`; writes `<app>.result-ast.yml` (gitignored) and fails on diff |
| `editors/vscode/test` | JSON-RPC framing, AST tree, byte-span conversion, application/quick-pick helpers, outside marker, lock debouncer |

## Status

Run `cargo test` for the current count; JavaScript tests run with
`node --test editors/vscode/test`. `clippy` and `fmt --check` should stay clean.

## AST snapshots

Each example application has `<app>.expected-ast.yml` next to its
`<app>-application.archol`. The snapshot test runs:

```bash
archol ast examples/applications/<path>/<app>-application.archol \
  --app <app-uri> --yaml
```

It writes the output to `<app>.result-ast.yml` (ignored by git) and compares
with the expected file. On failure the result file stays for diffing.

## Corpus

`tests/corpus.rs` walks `../../../examples` from the crate and fails if any
`.archol` file produces a parse diagnostic. It is the syntax acceptance test:
any new language construct enters a real example first and the parser afterward,
never the reverse.

## Fixtures

Small, intentionally wrong fixtures live in strings inside the tests (they do
not pollute `examples/`). To test the LSP manually, run:

```bash
cargo run -q --bin archol -- check ../examples --app example.com/app/backoffice-loja
cargo run -q --bin archol -- lock ../examples --app example.com/app/backoffice-loja
python3 scripts/smoke_lsp.py
```
