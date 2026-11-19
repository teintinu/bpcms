# Archol

**Archol** is the BPCMS modeling language (formerly called simply "DSL").
Every Archol source uses the **`.archol`** extension; the repository keeps the
examples in [`../examples/`](../examples) and the specification in
[`../DSL_REFERENCE.md`](../DSL_REFERENCE.md).

This directory contains the compiler, the LSP server, and the editor plugin.

## What is here

| Path | Contents |
| :--- | :--- |
| `crates/archol-core` | lexer, tokens, JSON AST, and parser |
| `crates/archol-analysis` | semantic check (enum, types, roles) and inverted link |
| `crates/archol-lsp` | Language Server Protocol server over stdio |
| `crates/archol-cli` | `archol` binary (`check`, `ast`, `tokens`, `links`, `lsp`) |
| `editors/vscode` | VSCode extension (LSP client + AST explorer), with no npm dependencies |
| `docs` | architecture, tokens, AST, LSP, and tests |
| `scripts` | `smoke_lsp.py` (initialize, hover, completion, AST, links) |
| `examples` | small fixtures used in tests and demonstrations |

Nothing is published or versioned: the language and the compiler are at the
**initial version**, evolving together. There are no tags, releases, or
compatibility guarantees to preserve.

## Requirements

- Stable Rust (`rustup`, `cargo`) — the project uses no crates beyond `serde`.
- Node.js (only to test the plugin; the extension has no dependencies).
- Windows, macOS, or Linux. All paths are handled with `/` internally.

## Build and test

```bash
cd archol
cargo build              # debug
cargo build --release    # final binary in target/release/archol
cargo test               # lexer, parser, check, link, and corpus tests
```

Compilation starts from an **application**. The commands below use the
backoffice app (its closure covers 172 files and 11 services):

```bash
cargo run -q --bin archol -- applications ../examples
cargo run -q --bin archol -- check ../examples --app example.com/app/backoffice-loja
cargo run -q --bin archol -- lock ../examples --app example.com/app/backoffice-loja
cargo run -q --bin archol -- links ../examples --app example.com/app/backoffice-loja > links.json
cargo run -q --bin archol -- ast ../examples/services/example.com/services/billing/billing-shapes.archol --app example.com/app/backoffice-loja
cargo run -q --bin archol -- view ../examples/services/example.com/services/billing/billing-shapes.archol --json
```

## `archol` binary

```text
archol check [dir] --app <uri|path> [--json]   compile one application closure
archol lint [dir] --app <uri|path> [--json]    check diagnostics (lint)
archol format [dir] [--check]                  uppercase pending keywords (format)
archol lock [dir] --app <uri|path>             create/prune <app>.lock (stable ids)
archol applications [dir] [--json]             list applications in a workspace
archol ast <file> [--app <uri|path>] [--json|--yaml] [--no-loc]  syntax AST, or annotated with ids
archol view <file> [--app <uri|path>] [--json|--pretty]  compact preview payload for editors
archol tokens <file>                           tokens (lexer debugging)
archol links [dir] --app <uri|path> [--json]   closure-only index for the builder
archol lsp                                    LSP server over stdio
```

`check` without `--app` exits with code 2 and lists the available applications.
`check` exits with code 1 when there is an error and prints
`file:line:column: severity code: message`. `lock` writes one identifier per
line (CSV `id,path`); the AST then carries stable `id`/`ref` values from the lock. Only used identifiers get ids; append `*` to a path to pin it. The plugin refreshes the lock on application selection and, with a 10 s debounce, after `.archol` edits.
`ast --yaml` exports the same AST as block-style YAML (strings are JSON-quoted), useful for snapshots. `--no-loc` removes every `loc`/`keywordLoc` property, keeping diffs small.
`format` uppercases `TODO`/`FIXME`/`REVIEW`/`OPTIMIZE`/`HACK` comment markers and
`--check` exits 1 when a file would change. `lint` runs the same diagnostics as
`check` (including pending due dates). `links` produces, among other things,
which artifacts require each `role`.
Files outside the application closure are treated as commented out (CLI and
LSP). See [`docs/identifiers.md`](docs/identifiers.md).

## Editor

One-command installation:

```bash
bash archol/scripts/install-vscode.sh
```

The script builds the server and installs the extension into
`~/.vscode/extensions/archol.archol-vscode-0.1.0`. Then just run **Developer:
Reload Window** and open a `.archol` file (with the repository folder as the
workspace). The extension starts the `archol lsp` binary automatically (or the
path configured in `archol.server.path`) and provides diagnostics, hover,
completion, symbols, definition, references, folding, and the **AST explorer**.
Details in [`editors/vscode/README.md`](editors/vscode/README.md).

### DeepSeek Harness

`editors/dsh/` is a DeepSeek Harness bundle that previews `.archol` files in
the Web right Sidebar: lexer-backed syntax highlighting, an artifact outline
that jumps to the declaration, and the file's diagnostics. It calls
`archol view <file> --json`, so build the binary first. See
[`editors/dsh/README.md`](editors/dsh/README.md).

## Coverage of this initial version

Already working: the full corpus lexer, tolerant parser (0 errors across 330
files) with a depth limit, JSON AST, check of enum values in `const`/`let` and
comparisons, enums resolved per service with `Alias.Type`, unknown
types/roles/references, duplication, `E_FILE_READ`, inverted link, LSP with
hover/completion/definition (including enum values)/references/symbols/folding,
and the VSCode plugin with the AST explorer.

Not yet implemented (documented in the specification, out of scope for this
round): `E_FILE_*` file path rules in the check, `semanticTokens`,
refactoring/rename, code actions, and deep validation of all process blocks. The
parser preserves those sections as `RawStatement`, so nothing breaks.

Robustness verified by tests: trailing `\` in strings, invalid UTF-8, deep
nesting, empty files, CRLF, and Unicode do not crash the CLI
(`archol-cli/tests/cli.rs`).

## Code principles

- Small files with a single responsibility; tokens organized under
  `crates/archol-core/src/token/` with tests per family.
- No heavy macros, no async, no runtime: a direct recursive parser.
- The AST is JSON (TypeScript style): every node has `kind` and `loc`.
- The check validates **values**, not just types: `const a: TaxRegimeCode = SIMPLES`
  and `regime == SIMPLES` are checked against the shape's `value`s.
- The link produces information the source does not have ready-made:
  `role -> artifacts`, `enum -> values`, `artifact -> references`.
