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
| `archol-cli/tests/cli.rs` | `--app` required, `applications`, `lock` create/stable/prune, application `ast`/`application`, `format --check`, pending diagnostics |
| `archol-cli/tests/application_snapshots.rs` | each resolved tree against `<app>.expected-ast.yml` with `archol application --no-loc --no-body`; writes `<app>.result-ast.yml` (gitignored) and fails on diff |
| `archol-cli/tests/application_ast.rs` | closed/typed `ArcholApplication` for every app, role/permission inverse, and the `reintroduce` merge in `loja-com-antifraude` |
| `editors/vscode/test` | JSON-RPC framing, AST tree, byte-span conversion, application/quick-pick helpers, outside marker, lock debouncer |

## Status

Run `cargo test` for the current count; JavaScript tests run with
`node --test editors/vscode/test`. `clippy` and `fmt --check` should stay clean.

## AST snapshots

Each example application has one snapshot next to its
`<app>-application.archol`:

- `<app>.expected-ast.yml` — the resolved `ArcholApplication` from
  `archol application examples --app <app-uri> --yaml --no-loc --no-body`.

There is no emitted `SourceFile` AST snapshot. The test writes
`<app>.result-ast.yml` (ignored by git) and compares it with the expected file.
On failure the result stays for diffing.

To refresh the application snapshots after an intentional change:

```bash
bash archol/scripts/test-update-snapshots.sh
```

To run the suite without touching snapshots:

```bash
bash archol/scripts/test.sh
```

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
