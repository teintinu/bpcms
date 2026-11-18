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

Running `check` over the real examples must finish without errors:

```bash
cargo run -q --bin archol -- check ../examples
cargo run -q --bin archol -- links ../examples > links.json
cargo run -q --bin archol -- ast ../examples/services/example.com/services/billing/billing-shapes.archol
```

## `archol` binary

```text
archol check [dir] [--json]    validates the workspace
archol ast <file> [--json]     prints the AST
archol tokens <file>           prints the tokens (lexer debugging)
archol links [dir] [--json]    inverted index for the builder
archol lsp                     LSP server over stdio
```

`check` exits with code 1 when there is an error and prints
`file:line:column: severity code: message`. `links` produces, among other
things, which artifacts require each `role`.

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
- The AST is JSON (TypeScript style): every node has `kind` and `span`.
- The check validates **values**, not just types: `const a: TaxRegimeCode = SIMPLES`
  and `regime == SIMPLES` are checked against the shape's `value`s.
- The link produces information the source does not have ready-made:
  `role -> artifacts`, `enum -> values`, `artifact -> references`.
