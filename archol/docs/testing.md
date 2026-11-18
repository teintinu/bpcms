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
| `archol-analysis/src/ids.rs` (inline tests) | identifier-safe base64 ids, JSONL roundtrip, pruning, duplicate rejection |
| `archol-analysis/src/application.rs` (inline tests) | application discovery, transitive imports, closure file set |
| `archol-lsp/src/resolve.rs` + `server/tests.rs` | field/enum definitions, app selection, declaration/usages by id |
| `archol-cli/tests/cli.rs` | `--app` required, `applications`, `lock` create/stable/prune, annotated `ast` |
| `editors/vscode/test` | JSON-RPC framing, AST tree, byte-span conversion, application/quick-pick helpers, outside marker |

## Status

Today: **56 Rust tests** (`cargo test`) and **40 JavaScript tests**
(`node --test editors/vscode/test`); `clippy` and `fmt --check` clean.

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
